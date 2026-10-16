use crate::{application::*, domain::*};
use async_trait::async_trait;
use serde_json::json;

pub struct DemoAdapter;
type DemoDataset = (Vec<Column>, Vec<serde_json::Map<String, serde_json::Value>>);
fn dataset(table: &str) -> Option<DemoDataset> {
    let fields: Vec<(&str, &str, bool)> = match table {
        "customers" => vec![
            ("id", "bigint", false),
            ("name", "text", false),
            ("email", "text", false),
            ("plan", "text", false),
            ("active", "boolean", false),
            ("created_at", "timestamp with time zone", false),
        ],
        "orders" => vec![
            ("id", "bigint", false),
            ("customer_id", "bigint", false),
            ("amount", "numeric", false),
            ("status", "text", false),
        ],
        "products" => vec![
            ("id", "bigint", false),
            ("name", "text", false),
            ("price", "numeric", false),
            ("description", "text", true),
        ],
        _ => return None,
    };
    let columns = fields
        .into_iter()
        .map(|(name, data_type, nullable)| Column {
            name: name.into(),
            data_type: data_type.into(),
            nullable,
            primary_key: name == "id",
        })
        .collect();
    let names = [
        "Amelia Brooks",
        "Oliver Chen",
        "Sofia Martinez",
        "Noah Williams",
        "Isabella Rossi",
        "Liam Anderson",
        "Mia Thompson",
        "Ethan Park",
        "Charlotte Wilson",
        "Lucas Meyer",
        "Ava Robinson",
        "Leo Nakamura",
    ];
    let rows = (1..=124).map(|i| {
        let name = names[(i - 1) % names.len()];
        let value = match table {
            "customers" => json!({"id": i.to_string(), "name": name, "email": format!("{}{}@example.com", name.split_whitespace().next().unwrap_or("user").to_lowercase(), i), "plan": (["Pro", "Team", "Free"][(i-1)%3]), "active": i%7 != 0, "created_at": format!("2026-09-{:02}T09:30:00Z", (i-1)%28+1)}),
            "orders" => json!({"id": i.to_string(), "customer_id": ((i-1)%12+1).to_string(), "amount": format!("{}.00", i*19), "status": (["completed", "pending", "refunded"][(i-1)%3])}),
            _ => json!({"id": i.to_string(), "name": format!("{} {}", ["Notebook", "Desk lamp", "Studio headphones"][(i-1)%3], i), "price": "49.00", "description": if i%3 == 0 { None } else { Some("A carefully considered everyday essential.") }}),
        };
        value.as_object().cloned().unwrap_or_default()
    }).collect();
    Some((columns, rows))
}
#[async_trait]
impl DatabaseAdapter for DemoAdapter {
    fn provider(&self) -> Provider {
        Provider {
            id: "demo",
            name: "Sample workspace",
        }
    }
    async fn test(&self, _: &ConnectionInput) -> AppResult<()> {
        Ok(())
    }
    async fn tables(&self, _: &ConnectionInput) -> AppResult<Vec<Table>> {
        Ok(["customers", "orders", "products"]
            .into_iter()
            .map(|name| Table {
                schema: "public".into(),
                name: name.into(),
                kind: "BASE TABLE".into(),
            })
            .collect())
    }
    async fn rows(&self, _: &ConnectionInput, query: &BrowseQuery) -> AppResult<TablePage> {
        let (page, page_size) = query.bounds().map_err(invalid)?;
        if query.schema != "public" {
            return Err(invalid("Unknown sample schema."));
        }
        let (columns, mut rows) =
            dataset(&query.table).ok_or_else(|| invalid("Unknown sample table."))?;
        if let Some(sort) = &query.sort {
            if !columns.iter().any(|c| &c.name == sort) {
                return Err(invalid("Choose an existing column to sort."));
            }
            rows.sort_by(|a, b| {
                let a = a[sort].to_string();
                let b = b[sort].to_string();
                let order = match (
                    a.trim_matches('"').parse::<f64>(),
                    b.trim_matches('"').parse::<f64>(),
                ) {
                    (Ok(a), Ok(b)) => a.total_cmp(&b),
                    _ => a.cmp(&b),
                };
                if query.descending.unwrap_or(false) {
                    order.reverse()
                } else {
                    order
                }
            });
        }
        let offset = (page * page_size) as usize;
        let has_more = rows.len() > offset + page_size as usize;
        Ok(TablePage {
            columns,
            rows: rows
                .into_iter()
                .skip(offset)
                .take(page_size as usize)
                .collect(),
            has_more,
            page,
            page_size,
        })
    }
}
