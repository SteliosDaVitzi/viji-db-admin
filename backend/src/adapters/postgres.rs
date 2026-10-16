use crate::{application::*, domain::*};
use async_trait::async_trait;
use sqlx::{
    Column as SqlxColumn, Connection, Executor, PgConnection, Row, SqlSafeStr, Statement, TypeInfo,
    postgres::{PgConnectOptions, PgSslMode},
};
use std::{str::FromStr, time::Duration};

pub struct PostgresAdapter;

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        // The infrastructure boundary replaces driver errors with safe messages.
        let message = match &error {
            sqlx::Error::Database(e) if e.code().as_deref() == Some("28P01") => {
                "Authentication failed. Check your username and password."
            }
            sqlx::Error::Database(e) if e.code().as_deref() == Some("42501") => {
                "This database user does not have permission to read that table."
            }
            sqlx::Error::Database(e) if e.code().as_deref() == Some("57014") => {
                "The query timed out. Try a smaller page."
            }
            _ => {
                "Database request failed. Check connection details, TLS settings, and read permissions."
            }
        };
        Self(ErrorKind::Upstream, message)
    }
}

pub fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn query_error(error: sqlx::Error) -> AppError {
    match &error {
        sqlx::Error::Database(e) => match e.code().as_deref() {
            Some("42601" | "42P01" | "42703" | "42702" | "42P18" | "42P02") => invalid(
                "Check the SQL syntax, table names, and column aliases. Run one SELECT query at a time.",
            ),
            Some("25006" | "0A000") => invalid("Only read-only SELECT queries are supported."),
            _ => error.into(),
        },
        _ => error.into(),
    }
}

async fn connect(input: &ConnectionInput) -> AppResult<PgConnection> {
    let options = PgConnectOptions::from_str("postgresql://")?
        .host(&input.host)
        .port(input.port)
        .database(&input.database)
        .username(&input.username)
        .password(&input.password)
        .ssl_mode(if input.ssl {
            PgSslMode::VerifyFull
        } else {
            PgSslMode::Disable
        })
        .application_name("webdb");
    tokio::time::timeout(
        Duration::from_secs(10),
        PgConnection::connect_with(&options),
    )
    .await
    .map_err(|_| {
        AppError(
            ErrorKind::Timeout,
            "Connection timed out. Check the host and port.",
        )
    })?
    .map_err(Into::into)
}

async fn begin(connection: &mut PgConnection) -> AppResult<sqlx::Transaction<'_, sqlx::Postgres>> {
    let mut tx = connection.begin().await?;
    sqlx::query("SET TRANSACTION READ ONLY")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL statement_timeout = '10s'")
        .execute(&mut *tx)
        .await?;
    sqlx::query("SET LOCAL lock_timeout = '2s'")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

#[async_trait]
impl DatabaseAdapter for PostgresAdapter {
    fn provider(&self) -> Provider {
        Provider {
            id: "postgres",
            name: "PostgreSQL",
        }
    }
    async fn test(&self, input: &ConnectionInput) -> AppResult<()> {
        let mut connection = connect(input).await?;
        let mut tx = begin(&mut connection).await?;
        sqlx::query("SELECT 1").execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
    async fn query(&self, input: &ConnectionInput, query: &SqlQuery) -> AppResult<TablePage> {
        let (page, page_size) = query.bounds().map_err(invalid)?;
        let mut connection = connect(input).await?;
        let mut tx = begin(&mut connection).await?;
        // Preparing a derived-table SELECT uses PostgreSQL's parser and single-statement
        // extended protocol. Commands cannot run here; execution remains READ ONLY.
        let source = format!("SELECT * FROM (\n{}\n) AS webdb_query", query.statement());
        let statement = (&mut *tx)
            .prepare(sqlx::AssertSqlSafe(source.as_str()).into_sql_str())
            .await
            .map_err(query_error)?;
        let mut names = std::collections::HashSet::new();
        let mut columns = Vec::new();
        let mut projection = Vec::new();
        for column in statement.columns() {
            if !names.insert(column.name().to_owned()) {
                return Err(invalid("Use unique column aliases for query results."));
            }
            let name = quote_identifier(column.name());
            let data_type = column.type_info().name();
            projection.push(
                if matches!(data_type, "INT8" | "NUMERIC" | "JSON" | "JSONB")
                    || data_type.ends_with("[]")
                {
                    format!("webdb_query.{name}::text AS {name}")
                } else {
                    format!("webdb_query.{name}")
                },
            );
            columns.push(Column {
                name: column.name().to_owned(),
                data_type: data_type.to_lowercase(),
                nullable: true,
                primary_key: false,
            });
        }
        if columns.is_empty() {
            return Err(invalid("Run a SELECT query that returns columns."));
        }
        let sql = format!(
            "SELECT to_jsonb(record) AS value FROM (SELECT {} FROM (\n{}\n) AS webdb_query LIMIT $1 OFFSET $2) AS record",
            projection.join(", "),
            query.statement()
        );
        // SQL is intentionally user-authored. Generated identifiers are quoted and
        // pagination values are bound; the read-only transaction is the write barrier.
        let records = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(i64::from(page_size) + 1)
            .bind(i64::from(page) * i64::from(page_size))
            .fetch_all(&mut *tx)
            .await
            .map_err(query_error)?;
        let has_more = records.len() > page_size as usize;
        let rows = records
            .into_iter()
            .take(page_size as usize)
            .map(|r| {
                let value: serde_json::Value = r.get("value");
                value.as_object().cloned().unwrap_or_default()
            })
            .collect();
        tx.rollback().await?;
        Ok(TablePage {
            columns,
            rows,
            has_more,
            page,
            page_size,
        })
    }
    async fn tables(&self, input: &ConnectionInput) -> AppResult<Vec<Table>> {
        let mut connection = connect(input).await?;
        let mut tx = begin(&mut connection).await?;
        let rows = sqlx::query("SELECT table_schema, table_name, table_type FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema') ORDER BY table_schema, table_name")
            .fetch_all(&mut *tx).await?;
        let tables = rows
            .iter()
            .map(|r| Table {
                schema: r.get("table_schema"),
                name: r.get("table_name"),
                kind: r.get("table_type"),
            })
            .collect();
        tx.commit().await?;
        Ok(tables)
    }
    async fn rows(&self, input: &ConnectionInput, query: &BrowseQuery) -> AppResult<TablePage> {
        let (page, page_size) = query.bounds().map_err(invalid)?;
        let mut connection = connect(input).await?;
        let mut tx = begin(&mut connection).await?;
        let metadata = sqlx::query(r#"SELECT c.column_name, c.data_type, c.is_nullable,
          EXISTS (SELECT 1 FROM information_schema.table_constraints tc JOIN information_schema.key_column_usage k
          ON tc.constraint_catalog=k.constraint_catalog AND tc.constraint_schema=k.constraint_schema AND tc.constraint_name=k.constraint_name
          WHERE tc.constraint_type='PRIMARY KEY' AND tc.table_schema=c.table_schema AND tc.table_name=c.table_name AND k.table_schema=c.table_schema AND k.table_name=c.table_name AND k.column_name=c.column_name) AS primary_key
          FROM information_schema.columns c WHERE c.table_schema=$1 AND c.table_name=$2 ORDER BY c.ordinal_position"#)
            .bind(&query.schema).bind(&query.table).fetch_all(&mut *tx).await?;
        if metadata.is_empty() {
            return Err(AppError(
                ErrorKind::NotFound,
                "Table not found or not accessible.",
            ));
        }
        let columns: Vec<Column> = metadata
            .iter()
            .map(|r| Column {
                name: r.get("column_name"),
                data_type: r.get("data_type"),
                nullable: r.get::<String, _>("is_nullable") == "YES",
                primary_key: r.get("primary_key"),
            })
            .collect();
        if let Some(sort) = &query.sort
            && !columns.iter().any(|c| &c.name == sort)
        {
            return Err(invalid("Choose an existing column to sort."));
        }
        let mut order: Vec<String> = columns
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| format!("source.{}", quote_identifier(&c.name)))
            .collect();
        if let Some(sort) = &query.sort {
            order.insert(
                0,
                format!(
                    "source.{} {} NULLS LAST",
                    quote_identifier(sort),
                    if query.descending.unwrap_or(false) {
                        "DESC"
                    } else {
                        "ASC"
                    }
                ),
            );
        }
        // Physical identifiers break ties, including across partitions, for unchanged data.
        let is_table: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema=$1 AND table_name=$2 AND table_type='BASE TABLE')").bind(&query.schema).bind(&query.table).fetch_one(&mut *tx).await?;
        if is_table {
            order.push("source.tableoid".into());
            order.push("source.ctid".into());
        }
        let order_by = if order.is_empty() {
            String::new()
        } else {
            format!(" ORDER BY {}", order.join(", "))
        };
        // Preserve exact 64-bit integers and decimals across JavaScript's number boundary.
        let projection = columns
            .iter()
            .map(|c| {
                let name = quote_identifier(&c.name);
                if matches!(
                    c.data_type.as_str(),
                    "bigint" | "numeric" | "decimal" | "ARRAY" | "json" | "jsonb"
                ) {
                    format!("source.{name}::text AS {name}")
                } else {
                    format!("source.{name}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "SELECT to_jsonb(record) AS value FROM (SELECT {projection} FROM {}.{} AS source{order_by} LIMIT $1 OFFSET $2) record",
            quote_identifier(&query.schema),
            quote_identifier(&query.table)
        );
        // Dynamic fragments are only quoted identifiers from validated metadata;
        // page size and offset remain bind parameters.
        let records = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .bind(i64::from(page_size) + 1)
            .bind(i64::from(page) * i64::from(page_size))
            .fetch_all(&mut *tx)
            .await?;
        let has_more = records.len() > page_size as usize;
        let rows = records
            .into_iter()
            .take(page_size as usize)
            .map(|r| {
                let value: serde_json::Value = r.get("value");
                value.as_object().cloned().unwrap_or_default()
            })
            .collect();
        tx.commit().await?;
        Ok(TablePage {
            columns,
            rows,
            has_more,
            page,
            page_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identifier_escaping() {
        assert_eq!(
            quote_identifier("a\"; DROP TABLE users;--"),
            "\"a\"\"; DROP TABLE users;--\""
        );
    }
}
