use crate::{application::*, domain::*};
use serde_json::{Map, Value};
use sqlparser::{
    ast::{ObjectType, Statement},
    dialect::Dialect,
    parser::Parser,
};
use std::collections::BTreeMap;

pub async fn bounded<T>(
    operation: impl std::future::Future<Output = AppResult<T>>,
) -> AppResult<T> {
    tokio::time::timeout(std::time::Duration::from_secs(15), operation).await.map_err(|_| AppError(ErrorKind::Timeout, "The change timed out. Refresh before retrying; the server may already have committed it."))?
}
pub fn identifier(name: &str, maximum: usize, quote: fn(&str) -> String) -> AppResult<String> {
    if name.is_empty() || name.len() > maximum || name.chars().any(char::is_control) {
        return Err(invalid(
            "Invalid object name. Check its length and control characters.",
        ));
    }
    Ok(quote(name))
}

pub fn statement(sql: &str, dialect: &dyn Dialect) -> AppResult<String> {
    if sql.trim().is_empty() || sql.len() > 12_000 || sql.contains("/*!") || sql.contains("/*M!") {
        return Err(invalid(
            "Enter one SQL change of at most 12,000 bytes, without executable comments.",
        ));
    }
    let statements = Parser::parse_sql(dialect, sql)
        .map_err(|_| invalid("Check the SQL change syntax. Run one statement at a time."))?;
    if statements.len() != 1
        || !matches!(
            &statements[0],
            Statement::CreateTable(_)
                | Statement::AlterTable(_)
                | Statement::CreateIndex(_)
                | Statement::Insert(_)
                | Statement::Update(_)
                | Statement::Delete(_)
                | Statement::Truncate(_)
                | Statement::Drop {
                    object_type: ObjectType::Table | ObjectType::Index,
                    ..
                }
        )
    {
        return Err(invalid(
            "Use one CREATE/ALTER/DROP TABLE, CREATE/DROP INDEX, TRUNCATE, INSERT, UPDATE, or DELETE statement. Use the dedicated database actions for databases.",
        ));
    }
    super::sql::canonical(&statements[0], dialect)
}

pub fn values(columns: &[Column], values: &BTreeMap<String, Option<String>>) -> AppResult<()> {
    for (name, value) in values {
        let column = columns
            .iter()
            .find(|c| &c.name == name)
            .ok_or_else(|| invalid("Unknown column. Refresh the table."))?;
        if column.read_only {
            return Err(invalid(
                "Omit generated, identity, and unsupported fields; the database maintains them.",
            ));
        }
        if value.as_ref().is_some_and(|s| s.len() > 12_000) {
            return Err(invalid("Field values must be at most 12,000 bytes."));
        }
    }
    Ok(())
}

pub fn keys<'a>(
    columns: &'a [Column],
    original: &Map<String, Value>,
) -> AppResult<Vec<(&'a Column, String)>> {
    if original.len() != columns.len() || columns.iter().any(|c| !original.contains_key(&c.name)) {
        return Err(invalid(
            "The table structure changed. Refresh before editing.",
        ));
    }
    let keys = columns
        .iter()
        .filter(|c| c.primary_key)
        .map(|column| {
            let value = original[&column.name]
                .as_str()
                .ok_or_else(|| invalid("Invalid primary key snapshot. Refresh the table."))?;
            Ok((column, value.to_owned()))
        })
        .collect::<AppResult<Vec<_>>>()?;
    if keys.is_empty() {
        return Err(invalid(
            "Updating or deleting records requires a primary key for this provider.",
        ));
    }
    Ok(keys)
}
pub fn unchanged(
    current: Option<&Map<String, Value>>,
    original: &Map<String, Value>,
) -> AppResult<()> {
    if current != Some(original) {
        return Err(invalid(
            "The record changed or was deleted. Refresh before editing; no changes were saved.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::{MsSqlDialect, MySqlDialect};
    #[test]
    fn provider_schema_examples_preserve_quoted_names() {
        let mysql=statement("CREATE TABLE `db`.`t``x` (id bigint PRIMARY KEY, amount decimal(24,4), computed bigint GENERATED ALWAYS AS (id+1) STORED) ENGINE=InnoDB",&MySqlDialect {}).unwrap();
        assert!(mysql.contains("`t``x`"));
        let mssql = statement(
            "CREATE TABLE [dbo].[t]]x] (id bigint IDENTITY(1,1) PRIMARY KEY, note nvarchar(50))",
            &MsSqlDialect {},
        )
        .unwrap();
        assert!(mssql.contains("[t]]x]"));
        assert!(
            statement(
                "CREATE TABLE t (id int, computed AS (id+1))",
                &MsSqlDialect {}
            )
            .is_err()
        );
    }
    #[test]
    fn changes_cannot_control_transactions_or_drop_databases() {
        for dialect in [&MySqlDialect {} as &dyn Dialect, &MsSqlDialect {}] {
            for sql in [
                "COMMIT",
                "BEGIN TRANSACTION",
                "DROP DATABASE master",
                "CREATE DATABASE x",
                "SELECT 1",
                "DELETE FROM t; COMMIT",
                "EXEC('DROP TABLE t')",
                "CREATE PROCEDURE p AS DELETE FROM t",
            ] {
                assert!(statement(sql, dialect).is_err(), "{sql}");
            }
            for sql in [
                "CREATE TABLE t (id int PRIMARY KEY)",
                "ALTER TABLE t ADD note varchar(20)",
                "DROP TABLE t",
                "INSERT INTO t (id) VALUES (1)",
                "UPDATE t SET id=2",
                "DELETE FROM t",
            ] {
                assert!(statement(sql, dialect).is_ok(), "{sql}");
            }
        }
    }
    #[test]
    fn stale_rows_and_malformed_keys_are_rejected() {
        let original = serde_json::json!({"id":"1","value":"before"})
            .as_object()
            .unwrap()
            .clone();
        assert!(unchanged(None, &original).is_err());
        assert!(
            unchanged(
                serde_json::json!({"id":"1","value":"after"}).as_object(),
                &original
            )
            .is_err()
        );
        assert!(unchanged(Some(&original), &original).is_ok());
        let columns = vec![Column {
            name: "id".into(),
            data_type: "int".into(),
            nullable: false,
            primary_key: false,
            read_only: false,
            default_value: None,
        }];
        assert!(keys(&columns, serde_json::json!({"id":"1"}).as_object().unwrap()).is_err());
    }
}
