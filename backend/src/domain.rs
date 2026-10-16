use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSettings {
    pub provider: String,
    pub database: String,
    pub properties: Map<String, Value>,
    pub settings: Vec<Value>,
    pub can_edit: bool,
}

#[derive(Deserialize)]
pub struct SettingChange {
    pub value: Option<String>,
    pub expected: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDatabaseSettings {
    pub database: String,
    pub confirmation: String,
    #[serde(default)]
    pub changes: std::collections::BTreeMap<String, SettingChange>,
    pub connection_limit: Option<i32>,
    pub expected_connection_limit: Option<i32>,
}

impl UpdateDatabaseSettings {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.confirmation != "APPLY" || self.database.is_empty() || self.database.len() > 128 {
            return Err("Review and save settings for the selected database.");
        }
        if self.changes.len() + usize::from(self.connection_limit.is_some()) > 50
            || (self.changes.is_empty() && self.connection_limit.is_none())
        {
            return Err("Save between 1 and 50 changes at a time.");
        }
        if self.connection_limit.is_some_and(|limit| limit < -1)
            || (self.connection_limit.is_some() && self.expected_connection_limit.is_none())
        {
            return Err("Enter a connection limit of -1 or greater.");
        }
        for (name, change) in &self.changes {
            if name.is_empty()
                || name.len() > 128
                || name.chars().any(char::is_control)
                || [change.value.as_deref(), change.expected.as_deref()]
                    .into_iter()
                    .flatten()
                    .any(|value| value.len() > 4096 || value.contains('\0'))
            {
                return Err("Invalid setting name or value. Values must be at most 4096 bytes.");
            }
        }
        Ok(())
    }
}

/// Writes are deliberately separate from browsing and read-only SQL.
#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum Mutation {
    Insert {
        schema: String,
        table: String,
        values: std::collections::BTreeMap<String, Option<String>>,
    },
    Update {
        schema: String,
        table: String,
        original: Map<String, Value>,
        values: std::collections::BTreeMap<String, Option<String>>,
    },
    Delete {
        schema: String,
        table: String,
        original: Map<String, Value>,
    },
    CreateDatabase {
        name: String,
    },
    DropDatabase {
        name: String,
    },
    DropTable {
        schema: String,
        table: String,
    },
    Execute {
        sql: String,
    },
}

#[derive(Deserialize)]
pub struct MutationRequest {
    pub confirmation: String,
    pub mutation: Mutation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub affected_rows: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionInput {
    pub provider: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl: bool,
    #[serde(default)]
    pub all_databases: bool,
}

impl ConnectionInput {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.provider == "demo" {
            return Ok(());
        }
        if self.host.is_empty()
            || self.host.len() > 253
            || self
                .host
                .chars()
                .any(|c| c.is_control() || "/\\;".contains(c))
        {
            return Err("Enter a valid database host.");
        }
        if self.port == 0 {
            return Err("Enter a port between 1 and 65535.");
        }
        if (!self.all_databases && self.database.trim().is_empty())
            || self.username.trim().is_empty()
            || self.database.len() > 128
            || self.username.len() > 128
            || self.password.len() > 4096
        {
            return Err("Enter a database and username of at most 128 characters.");
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct SavedConnection {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct SaveConnection {
    pub name: String,
}

#[derive(Deserialize)]
pub struct SelectDatabase {
    pub database: String,
}

#[derive(Default, Deserialize)]
pub struct DatabaseTablesQuery {
    pub database: Option<String>,
}

#[derive(Serialize)]
pub struct Provider {
    pub id: &'static str,
    pub name: &'static str,
}
#[derive(Clone, Serialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub columns: Vec<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub read_only: bool,
    pub default_value: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablePage {
    pub columns: Vec<Column>,
    pub rows: Vec<Map<String, Value>>,
    pub has_more: bool,
    pub page: u32,
    pub page_size: u32,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowseQuery {
    pub schema: String,
    pub table: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort: Option<String>,
    pub descending: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SqlQuery {
    pub sql: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}
impl SqlQuery {
    pub fn bounds(&self) -> Result<(u32, u32), &'static str> {
        let page = self.page.unwrap_or(0);
        let size = self.page_size.unwrap_or(50);
        if self.sql.trim().is_empty() || self.sql.len() > 12_000 {
            return Err("Enter a query of at most 12,000 bytes.");
        }
        if page > 100_000 || !(1..=1000).contains(&size) {
            return Err("Invalid query pagination parameters.");
        }
        Ok((page, size))
    }
    pub fn statement(&self) -> &str {
        self.sql.trim().trim_end_matches(';').trim_end()
    }
}
impl BrowseQuery {
    pub fn bounds(&self) -> Result<(u32, u32), &'static str> {
        let page = self.page.unwrap_or(0);
        let size = self.page_size.unwrap_or(50);
        if page > 100_000
            || !(1..=1000).contains(&size)
            || self.schema.is_empty()
            || self.table.is_empty()
            || self.schema.len() > 128
            || self.table.len() > 128
        {
            return Err("Invalid table or pagination parameters.");
        }
        Ok((page, size))
    }
}
