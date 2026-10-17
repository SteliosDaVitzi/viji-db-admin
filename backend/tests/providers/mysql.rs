#[tokio::test]
#[ignore = "Requires WEBDB_TEST_MYSQL_CONNECTION_JSON naming a disposable MySQL database"]
async fn browse() {
    super::super::browse(
        &dbsaur::adapters::mysql::MySqlAdapter,
        "WEBDB_TEST_MYSQL_CONNECTION_JSON",
        "mysql",
        "SELECT CAST(9007199254740993 AS DECIMAL(20,0)) AS exact_value, NULL AS nullable_value",
    )
    .await;
}

#[tokio::test]
#[ignore = "Requires WEBDB_TEST_MYSQL_CONNECTION_JSON naming a disposable database with DDL permissions"]
async fn writes() {
    super::super::writes(
        &dbsaur::adapters::mysql::MySqlAdapter,
        "WEBDB_TEST_MYSQL_CONNECTION_JSON",
        "mysql",
    )
    .await;
}
