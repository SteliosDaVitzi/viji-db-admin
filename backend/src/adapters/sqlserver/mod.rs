mod write;
use crate::{application::*, domain::*};
use async_trait::async_trait;
use std::{collections::BTreeMap, time::Duration};
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, Row};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type SqlClient = Client<Compat<TcpStream>>;
pub struct SqlServerAdapter;
fn driver_error(_: tiberius::error::Error) -> AppError {
    AppError(
        ErrorKind::Upstream,
        "SQL Server request failed. Check connection details, TLS, permissions, SQL syntax, and unique column aliases.",
    )
}
async fn connect(input: &ConnectionInput) -> AppResult<SqlClient> {
    connect_mode(input, true).await
}
async fn connect_mode(input: &ConnectionInput, read_only: bool) -> AppResult<SqlClient> {
    let mut config = Config::new();
    config.host(&input.host);
    config.port(input.port);
    config.database(&input.database);
    config.authentication(AuthMethod::sql_server(&input.username, &input.password));
    config.application_name("webdb");
    config.encryption(if input.ssl {
        EncryptionLevel::Required
    } else {
        EncryptionLevel::NotSupported
    });
    config.command_timeout(Some(Duration::from_secs(10)));
    // ApplicationIntent is a routing hint; SQL validation below is still required.
    config.readonly(read_only);
    let mut client = tokio::time::timeout(Duration::from_secs(10), async {
        let tcp = TcpStream::connect(config.get_addr()).await.map_err(|_| {
            AppError(
                ErrorKind::Upstream,
                "Cannot reach SQL Server. Check the host and TCP port.",
            )
        })?;
        tcp.set_nodelay(true).map_err(|_| {
            AppError(
                ErrorKind::Upstream,
                "Cannot configure SQL Server connection.",
            )
        })?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(driver_error)
    })
    .await
    .map_err(|_| AppError(ErrorKind::Timeout, "SQL Server connection timed out."))??;
    client
        .simple_query("SET IMPLICIT_TRANSACTIONS OFF; SET LOCK_TIMEOUT 2000; SET NOCOUNT ON;")
        .await
        .map_err(driver_error)?
        .into_results()
        .await
        .map_err(driver_error)?;
    Ok(client)
}
pub fn quote_identifier(name: &str) -> String {
    format!("[{}]", name.replace(']', "]]"))
}
fn text(row: &Row, name: &str) -> AppResult<String> {
    row.try_get::<&str, _>(name)
        .map_err(driver_error)?
        .map(str::to_owned)
        .ok_or_else(|| invalid("Database metadata is unavailable."))
}
async fn metadata(client: &mut SqlClient, schema: &str, table: &str) -> AppResult<Vec<Column>> {
    let rows = client.query("SELECT c.COLUMN_NAME,c.DATA_TYPE,c.IS_NULLABLE,c.COLUMN_DEFAULT,CAST(CASE WHEN COALESCE(COLUMNPROPERTY(OBJECT_ID(QUOTENAME(c.TABLE_SCHEMA)+'.'+QUOTENAME(c.TABLE_NAME)),c.COLUMN_NAME,'IsIdentity'),0)=1 OR COALESCE(COLUMNPROPERTY(OBJECT_ID(QUOTENAME(c.TABLE_SCHEMA)+'.'+QUOTENAME(c.TABLE_NAME)),c.COLUMN_NAME,'IsComputed'),0)=1 OR COALESCE(COLUMNPROPERTY(OBJECT_ID(QUOTENAME(c.TABLE_SCHEMA)+'.'+QUOTENAME(c.TABLE_NAME)),c.COLUMN_NAME,'GeneratedAlwaysType'),0)<>0 OR c.DATA_TYPE IN ('timestamp','rowversion','geometry','geography','hierarchyid','sql_variant') THEN 1 ELSE 0 END AS bit) AS read_only,CAST(CASE WHEN EXISTS (SELECT 1 FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE k ON tc.CONSTRAINT_CATALOG=k.CONSTRAINT_CATALOG AND tc.CONSTRAINT_SCHEMA=k.CONSTRAINT_SCHEMA AND tc.CONSTRAINT_NAME=k.CONSTRAINT_NAME WHERE tc.CONSTRAINT_TYPE='PRIMARY KEY' AND tc.TABLE_SCHEMA=c.TABLE_SCHEMA AND tc.TABLE_NAME=c.TABLE_NAME AND k.COLUMN_NAME=c.COLUMN_NAME) THEN 1 ELSE 0 END AS bit) AS primary_key FROM INFORMATION_SCHEMA.COLUMNS c WHERE c.TABLE_SCHEMA=@P1 AND c.TABLE_NAME=@P2 ORDER BY c.ORDINAL_POSITION", &[&schema,&table]).await.map_err(driver_error)?.into_first_result().await.map_err(driver_error)?;
    rows.iter()
        .map(|row| {
            Ok(Column {
                name: text(row, "COLUMN_NAME")?,
                data_type: text(row, "DATA_TYPE")?,
                nullable: text(row, "IS_NULLABLE")? == "YES",
                primary_key: row
                    .try_get::<bool, _>("primary_key")
                    .map_err(driver_error)?
                    .unwrap_or(false),
                read_only: row
                    .try_get::<bool, _>("read_only")
                    .map_err(driver_error)?
                    .unwrap_or(true),
                default_value: row
                    .try_get::<&str, _>("COLUMN_DEFAULT")
                    .map_err(driver_error)?
                    .map(str::to_owned),
            })
        })
        .collect()
}
fn projection(columns: &[Column]) -> String {
    columns
        .iter()
        .map(|column| {
            let name = quote_identifier(&column.name);
            let binary = ["binary", "image", "timestamp", "rowversion"]
                .iter()
                .any(|t| column.data_type.contains(t));
            let style = if binary || matches!(column.data_type.as_str(), "money" | "smallmoney") {
                2
            } else if column.data_type.starts_with("float") || column.data_type.starts_with("real")
            {
                3
            } else {
                126
            };
            let expression = if binary {
                format!("CONVERT(varbinary(max), source.{name})")
            } else {
                format!("source.{name}")
            };
            format!("CONVERT(nvarchar(max), {expression}, {style}) AS {name}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}
async fn page(
    client: &mut SqlClient,
    source: &str,
    columns: Vec<Column>,
    order: &str,
    number: u32,
    size: u32,
) -> AppResult<TablePage> {
    let sql = format!(
        "SELECT {} FROM {source} AS source ORDER BY {order} OFFSET @P1 ROWS FETCH NEXT @P2 ROWS ONLY",
        projection(&columns)
    );
    let offset = i64::from(number) * i64::from(size);
    let limit = i64::from(size) + 1;
    let records = client
        .query(sql, &[&offset, &limit])
        .await
        .map_err(driver_error)?
        .into_first_result()
        .await
        .map_err(driver_error)?;
    let has_more = records.len() > size as usize;
    let rows = records
        .into_iter()
        .take(size as usize)
        .map(|row| {
            columns
                .iter()
                .enumerate()
                .map(|(index, column)| {
                    Ok((
                        column.name.clone(),
                        row.try_get::<&str, _>(index)
                            .map_err(driver_error)?
                            .map_or(serde_json::Value::Null, |s| s.into()),
                    ))
                })
                .collect::<AppResult<_>>()
        })
        .collect::<AppResult<_>>()?;
    Ok(TablePage {
        columns,
        rows,
        has_more,
        page: number,
        page_size: size,
    })
}
#[async_trait]
impl DatabaseAdapter for SqlServerAdapter {
    async fn mutate(
        &self,
        input: &ConnectionInput,
        mutation: &Mutation,
    ) -> AppResult<MutationResult> {
        super::write::bounded(write::mutate(input, mutation)).await
    }
    fn maintenance_database(&self) -> Option<&'static str> {
        Some("master")
    }
    fn provider(&self) -> Provider {
        Provider {
            id: "sqlserver",
            name: "SQL Server",
        }
    }
    async fn test(&self, input: &ConnectionInput) -> AppResult<()> {
        let mut c = connect(input).await?;
        c.query("SELECT 1", &[])
            .await
            .map_err(driver_error)?
            .into_results()
            .await
            .map_err(driver_error)?;
        Ok(())
    }
    async fn databases(&self, input: &ConnectionInput) -> AppResult<Vec<String>> {
        let mut c = connect(input).await?;
        let rows=c.query("SELECT name FROM sys.databases WHERE state=0 AND HAS_DBACCESS(name)=1 AND name NOT IN ('master','model','msdb','tempdb') ORDER BY name",&[]).await.map_err(driver_error)?.into_first_result().await.map_err(driver_error)?;
        rows.iter().map(|row| text(row, "name")).collect()
    }
    async fn tables(&self, input: &ConnectionInput) -> AppResult<Vec<Table>> {
        let mut c = connect(input).await?;
        let records=c.query("SELECT t.TABLE_SCHEMA,t.TABLE_NAME,t.TABLE_TYPE,c.COLUMN_NAME FROM INFORMATION_SCHEMA.TABLES t LEFT JOIN INFORMATION_SCHEMA.COLUMNS c ON c.TABLE_SCHEMA=t.TABLE_SCHEMA AND c.TABLE_NAME=t.TABLE_NAME ORDER BY t.TABLE_SCHEMA,t.TABLE_NAME,c.ORDINAL_POSITION",&[]).await.map_err(driver_error)?.into_first_result().await.map_err(driver_error)?;
        let mut tables = BTreeMap::<(String, String), Table>::new();
        for row in &records {
            let schema = text(row, "TABLE_SCHEMA")?;
            let name = text(row, "TABLE_NAME")?;
            let table = tables
                .entry((schema.clone(), name.clone()))
                .or_insert(Table {
                    schema,
                    name,
                    kind: text(row, "TABLE_TYPE")?,
                    columns: vec![],
                });
            if let Some(name) = row
                .try_get::<&str, _>("COLUMN_NAME")
                .map_err(driver_error)?
            {
                table.columns.push(name.into());
            }
        }
        Ok(tables.into_values().collect())
    }
    async fn rows(&self, input: &ConnectionInput, query: &BrowseQuery) -> AppResult<TablePage> {
        let (number, size) = query.bounds().map_err(invalid)?;
        let mut c = connect(input).await?;
        let columns = metadata(&mut c, &query.schema, &query.table).await?;
        if columns.is_empty() {
            return Err(AppError(
                ErrorKind::NotFound,
                "Table not found or not accessible.",
            ));
        }
        let mut order = columns
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| format!("source.{}", quote_identifier(&c.name)))
            .collect::<Vec<_>>();
        if let Some(sort) = &query.sort {
            if !columns.iter().any(|c| &c.name == sort) {
                return Err(invalid("Choose an existing column to sort."));
            }
            order.insert(
                0,
                format!(
                    "source.{} {}",
                    quote_identifier(sort),
                    if query.descending.unwrap_or(false) {
                        "DESC"
                    } else {
                        "ASC"
                    }
                ),
            );
        }
        let order = if order.is_empty() {
            "(SELECT NULL)".into()
        } else {
            order.join(", ")
        };
        page(
            &mut c,
            &format!(
                "{}.{}",
                quote_identifier(&query.schema),
                quote_identifier(&query.table)
            ),
            columns,
            &order,
            number,
            size,
        )
        .await
    }
    async fn query(&self, input: &ConnectionInput, query: &SqlQuery) -> AppResult<TablePage> {
        let (number, size) = query.bounds().map_err(invalid)?;
        let sql = super::read_query::select(&query.sql, &sqlparser::dialect::MsSqlDialect {})?;
        let source = format!("({sql})");
        let describe = format!("SELECT * FROM {source} AS source");
        let mut c = connect(input).await?;
        // Describing the derived SELECT validates it with SQL Server without executing it.
        let records=c.query("SELECT name,system_type_name,is_nullable,error_number FROM sys.dm_exec_describe_first_result_set(@P1,NULL,0) WHERE is_hidden=0 OR error_number IS NOT NULL ORDER BY column_ordinal",&[&describe]).await.map_err(driver_error)?.into_first_result().await.map_err(driver_error)?;
        let mut names = std::collections::HashSet::new();
        let mut columns = Vec::new();
        for row in &records {
            if row
                .try_get::<i32, _>("error_number")
                .map_err(driver_error)?
                .is_some()
            {
                return Err(invalid(
                    "Check SQL Server SELECT syntax and aliases. Use TOP or OFFSET with an inner ORDER BY; CTEs are not supported here.",
                ));
            }
            let name = text(row, "name")?;
            if !names.insert(name.clone()) {
                return Err(invalid("Use unique column aliases for query results."));
            }
            columns.push(Column {
                name,
                data_type: text(row, "system_type_name")?,
                nullable: row
                    .try_get::<bool, _>("is_nullable")
                    .map_err(driver_error)?
                    .unwrap_or(true),
                primary_key: false,
                read_only: true,
                default_value: None,
            });
        }
        if columns.is_empty() {
            return Err(invalid("Run a SELECT query that returns named columns."));
        }
        page(&mut c, &source, columns, "(SELECT NULL)", number, size).await
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn identifiers_are_quoted() {
        assert_eq!(
            super::quote_identifier("x]; DROP TABLE t;--"),
            "[x]]; DROP TABLE t;--]"
        );
    }
}
