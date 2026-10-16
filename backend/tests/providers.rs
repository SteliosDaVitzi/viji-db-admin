//! Explicit opt-in smoke tests. No database is created or selected implicitly.
mod providers {
    pub mod mariadb;
    pub mod mysql;
    pub mod sqlserver;
}

async fn browse(
    adapter: &dyn webdb::application::DatabaseAdapter,
    variable: &str,
    provider: &str,
    sql: &str,
) {
    let json =
        std::env::var(variable).expect("Set an explicit disposable database connection JSON");
    let input: webdb::domain::ConnectionInput =
        serde_json::from_str(&json).expect("Valid ConnectionInput JSON");
    assert_eq!(input.provider, provider);
    assert!(
        !input.all_databases,
        "Use an explicitly named disposable database"
    );
    input.validate().unwrap();
    adapter.test(&input).await.unwrap();
    adapter.tables(&input).await.unwrap();
    let page = adapter
        .query(
            &input,
            &webdb::domain::SqlQuery {
                sql: sql.into(),
                page: Some(0),
                page_size: Some(1),
            },
        )
        .await
        .unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0]["exact_value"], "9007199254740993");
    assert_eq!(page.rows[0]["nullable_value"], serde_json::Value::Null);
    assert!(page.columns.iter().all(|column| column.read_only));
    // The write guard must reject these without reaching the database.
    for sql in [
        "DELETE FROM webdb_should_never_exist",
        "SELECT 1 AS n; DROP TABLE webdb_should_never_exist",
        "SELECT 1 AS n INTO webdb_should_never_exist",
    ] {
        assert!(
            adapter
                .query(
                    &input,
                    &webdb::domain::SqlQuery {
                        sql: sql.into(),
                        page: None,
                        page_size: None
                    }
                )
                .await
                .is_err()
        );
    }
    let result = adapter
        .mutate(
            &input,
            &webdb::domain::Mutation::Execute {
                sql: "COMMIT".into(),
            },
        )
        .await;
    assert!(result.is_err());
}

use std::collections::BTreeMap;
use webdb::{application::DatabaseAdapter, domain::*};
fn values(items: &[(&str, Option<&str>)]) -> BTreeMap<String, Option<String>> {
    items
        .iter()
        .map(|(key, value)| ((*key).into(), value.map(str::to_owned)))
        .collect()
}
async fn writes(adapter: &dyn DatabaseAdapter, variable: &str, provider: &str) {
    let input: ConnectionInput = serde_json::from_str(
        &std::env::var(variable).expect("Set a disposable database connection JSON"),
    )
    .unwrap();
    assert_eq!(input.provider, provider);
    assert!(
        !input.all_databases,
        "Name an explicitly disposable database"
    );
    input.validate().unwrap();
    let schema = if provider == "sqlserver" {
        "dbo".into()
    } else {
        input.database.clone()
    };
    let quote: fn(&str) -> String = if provider == "sqlserver" {
        webdb::adapters::sqlserver::quote_identifier
    } else {
        webdb::adapters::mysql::quote_identifier
    };
    let name = format!(
        "webdb_write_{}{}",
        uuid::Uuid::new_v4().simple(),
        if provider == "sqlserver" { "]" } else { "`" }
    );
    let qualified = format!("{}.{}", quote(&schema), quote(&name));
    let parent = format!("webdb_parent_{}", uuid::Uuid::new_v4().simple());
    let parent_qualified = format!("{}.{}", quote(&schema), quote(&parent));
    let defaults = format!("webdb_defaults_{}", uuid::Uuid::new_v4().simple());
    let defaults_qualified = format!("{}.{}", quote(&schema), quote(&defaults));
    let no_key = format!("webdb_nokey_{}", uuid::Uuid::new_v4().simple());
    let no_key_qualified = format!("{}.{}", quote(&schema), quote(&no_key));
    let generated = if provider == "sqlserver" {
        "computed AS (id+1)"
    } else {
        "computed bigint GENERATED ALWAYS AS (id+1) STORED"
    };
    let engine = if provider == "sqlserver" {
        ""
    } else {
        " ENGINE=InnoDB"
    };
    let identity = if provider == "sqlserver" {
        "bigint IDENTITY(1,1)"
    } else {
        "bigint AUTO_INCREMENT"
    };
    let statements = [
        format!("CREATE TABLE {parent_qualified} (id bigint PRIMARY KEY){engine}"),
        format!("INSERT INTO {parent_qualified} (id) VALUES (1)"),
        format!(
            "CREATE TABLE {qualified} (id bigint PRIMARY KEY, parent_id bigint, amount decimal(24,4) CHECK(amount>0), note varchar(5) NOT NULL DEFAULT 'hello', payload varchar(100), data varbinary(8), {generated}, FOREIGN KEY(parent_id) REFERENCES {parent_qualified}(id)){engine}"
        ),
        format!(
            "CREATE TABLE {defaults_qualified} (id {identity} PRIMARY KEY, note varchar(5) NOT NULL DEFAULT 'hello'){engine}"
        ),
        format!("CREATE TABLE {no_key_qualified} (note varchar(5)){engine}"),
    ];
    let result = async {
        for sql in statements {
            if provider == "sqlserver" && sql.contains("computed AS") {
                // Set up an existing computed column independently of the application's
                // deliberately limited SQL-change parser.
                sqlserver_fixture_sql(&input, &sql).await;
            } else {
                adapter
                    .mutate(&input, &Mutation::Execute { sql })
                    .await
                    .unwrap();
            }
        }
        let insert = |fields| Mutation::Insert {
            schema: schema.clone(),
            table: name.clone(),
            values: fields,
        };
        let query = BrowseQuery {
            schema: schema.clone(),
            table: name.clone(),
            ..Default::default()
        };
        adapter
            .mutate(
                &input,
                &insert(values(&[
                    ("id", Some("9007199254740993")),
                    ("parent_id", Some("1")),
                    ("amount", Some("12345678901234.1234")),
                    ("payload", Some("{\"n\":9007199254740993}")),
                    ("data", Some("00aaff")),
                ])),
            )
            .await
            .unwrap();
        adapter
            .mutate(
                &input,
                &insert(values(&[
                    ("id", Some("9007199254740994")),
                    ("parent_id", Some("1")),
                    ("amount", Some("1")),
                    ("note", Some("other")),
                ])),
            )
            .await
            .unwrap();
        let page = adapter.rows(&input, &query).await.unwrap();
        assert!(
            page.columns
                .iter()
                .find(|c| c.name == "computed")
                .unwrap()
                .read_only
        );
        assert!(
            !page
                .columns
                .iter()
                .find(|c| c.name == "note")
                .unwrap()
                .read_only
        );
        let original = page.rows[0].clone();
        assert_eq!(original["amount"], "12345678901234.1234");
        assert_eq!(original["note"], "hello");
        assert_eq!(original["data"].as_str().unwrap().to_lowercase(), "00aaff");
        for (column, value) in [
            ("parent_id", Some("999")),
            ("amount", Some("-1")),
            ("note", None),
            ("note", Some("toolong")),
            ("computed", Some("9")),
            ("not_a_column", Some("bad")),
        ] {
            let error = adapter
                .mutate(
                    &input,
                    &Mutation::Update {
                        schema: schema.clone(),
                        table: name.clone(),
                        original: original.clone(),
                        values: values(&[(column, value)]),
                    },
                )
                .await;
            assert!(error.is_err(), "Must reject {column}={value:?}");
            assert_eq!(
                adapter.rows(&input, &query).await.unwrap().rows[0],
                original
            );
        }
        adapter
            .mutate(
                &input,
                &Mutation::Update {
                    schema: schema.clone(),
                    table: name.clone(),
                    original: original.clone(),
                    values: values(&[
                        ("note", Some("';--")),
                        ("payload", None),
                        ("data", Some("")),
                    ]),
                },
            )
            .await
            .unwrap();
        let mut changed = adapter.rows(&input, &query).await.unwrap().rows.remove(0);
        let other = adapter.rows(&input, &query).await.unwrap().rows.remove(1);
        assert_eq!(other["note"], "other");
        adapter
            .mutate(
                &input,
                &Mutation::Delete {
                    schema: schema.clone(),
                    table: name.clone(),
                    original: other,
                },
            )
            .await
            .unwrap();
        adapter
            .mutate(
                &input,
                &Mutation::Update {
                    schema: schema.clone(),
                    table: name.clone(),
                    original: changed.clone(),
                    values: values(&[("id", Some("9007199254740995"))]),
                },
            )
            .await
            .unwrap();
        changed = adapter.rows(&input, &query).await.unwrap().rows.remove(0);
        assert_eq!(changed["id"], "9007199254740995");
        assert_eq!(changed["note"], "';--");
        assert_eq!(changed["payload"], serde_json::Value::Null);
        assert_eq!(changed["data"], "");
        assert!(
            adapter
                .mutate(
                    &input,
                    &Mutation::Update {
                        schema: schema.clone(),
                        table: name.clone(),
                        original: original.clone(),
                        values: values(&[("note", Some("stale"))])
                    }
                )
                .await
                .is_err()
        );
        assert!(
            adapter
                .mutate(
                    &input,
                    &Mutation::Delete {
                        schema: schema.clone(),
                        table: name.clone(),
                        original
                    }
                )
                .await
                .is_err()
        );
        adapter
            .mutate(
                &input,
                &Mutation::Delete {
                    schema: schema.clone(),
                    table: name.clone(),
                    original: changed,
                },
            )
            .await
            .unwrap();
        assert!(adapter.rows(&input, &query).await.unwrap().rows.is_empty());
        adapter
            .mutate(
                &input,
                &Mutation::Insert {
                    schema: schema.clone(),
                    table: defaults.clone(),
                    values: BTreeMap::new(),
                },
            )
            .await
            .unwrap();
        let default_page = adapter
            .rows(
                &input,
                &BrowseQuery {
                    schema: schema.clone(),
                    table: defaults.clone(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(default_page.rows[0]["note"], "hello");
        assert!(
            default_page
                .columns
                .iter()
                .find(|c| c.name == "id")
                .unwrap()
                .read_only
        );
        adapter
            .mutate(
                &input,
                &Mutation::Insert {
                    schema: schema.clone(),
                    table: no_key.clone(),
                    values: values(&[("note", Some("a"))]),
                },
            )
            .await
            .unwrap();
        let no_key_row = adapter
            .rows(
                &input,
                &BrowseQuery {
                    schema: schema.clone(),
                    table: no_key.clone(),
                    ..Default::default()
                },
            )
            .await
            .unwrap()
            .rows
            .remove(0);
        assert!(
            adapter
                .mutate(
                    &input,
                    &Mutation::Delete {
                        schema: schema.clone(),
                        table: no_key.clone(),
                        original: no_key_row
                    }
                )
                .await
                .is_err()
        );
        // Browsing remains read only even after an explicit write connection was used.
        assert!(
            adapter
                .query(
                    &input,
                    &SqlQuery {
                        sql: format!("DELETE FROM {qualified}"),
                        page: None,
                        page_size: None
                    }
                )
                .await
                .is_err()
        );
    };
    // Poll under catch_unwind so failed assertions still tear down the fixtures.
    use std::{
        future::Future,
        panic::{AssertUnwindSafe, catch_unwind},
        task::Poll,
    };
    let mut result = Box::pin(result);
    let outcome = std::future::poll_fn(|cx| {
        match catch_unwind(AssertUnwindSafe(|| result.as_mut().poll(cx))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(())) => Poll::Ready(Ok(())),
            Err(panic) => Poll::Ready(Err(panic)),
        }
    })
    .await;
    drop(result);
    let mut cleanup_errors = Vec::new();
    for table in [name, defaults, no_key, parent] {
        if let Err(error) = adapter
            .mutate(
                &input,
                &Mutation::DropTable {
                    schema: schema.clone(),
                    table,
                },
            )
            .await
        {
            cleanup_errors.push(error);
        }
    }
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
    assert!(
        cleanup_errors.is_empty(),
        "Disposable fixture cleanup failed"
    );
}

async fn sqlserver_fixture_sql(input: &ConnectionInput, sql: &str) {
    use tokio_util::compat::TokioAsyncWriteCompatExt;
    let mut config = tiberius::Config::new();
    config.host(&input.host);
    config.port(input.port);
    config.database(&input.database);
    config.authentication(tiberius::AuthMethod::sql_server(
        &input.username,
        &input.password,
    ));
    config.encryption(if input.ssl {
        tiberius::EncryptionLevel::Required
    } else {
        tiberius::EncryptionLevel::NotSupported
    });
    config.command_timeout(Some(std::time::Duration::from_secs(10)));
    let tcp = tokio::net::TcpStream::connect(config.get_addr())
        .await
        .unwrap_or_else(|_| panic!("SQL Server fixture connection failed"));
    let mut client = tiberius::Client::connect(config, tcp.compat_write())
        .await
        .unwrap_or_else(|_| panic!("SQL Server fixture login failed"));
    // All SQL is test-authored and targets randomly named fixtures in the explicit
    // disposable database; credentials and raw driver errors are never logged.
    assert!(
        client.execute(sql, &[]).await.is_ok(),
        "SQL Server fixture setup failed"
    );
}
