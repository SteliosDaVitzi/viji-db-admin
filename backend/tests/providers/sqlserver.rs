#[tokio::test]
#[ignore = "Requires WEBDB_TEST_SQLSERVER_CONNECTION_JSON naming a disposable SQL Server database"]
async fn browse() {
    super::super::browse(&webdb::adapters::sqlserver::SqlServerAdapter, "WEBDB_TEST_SQLSERVER_CONNECTION_JSON", "sqlserver", "SELECT CAST(9007199254740993 AS decimal(20,0)) AS exact_value, CAST(NULL AS nvarchar(20)) AS nullable_value").await;
}

#[tokio::test]
#[ignore = "Requires WEBDB_TEST_SQLSERVER_CONNECTION_JSON naming a disposable database with DDL permissions"]
async fn writes() {
    super::super::writes(
        &webdb::adapters::sqlserver::SqlServerAdapter,
        "WEBDB_TEST_SQLSERVER_CONNECTION_JSON",
        "sqlserver",
    )
    .await;
}
