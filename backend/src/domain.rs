use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseSettings {
    pub provider: String,
    pub database: String,
    pub properties: Map<String, Value>,
    pub settings: Vec<Value>,
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
        if page > 100_000 || !(1..=100).contains(&size) {
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
            || !(1..=100).contains(&size)
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
