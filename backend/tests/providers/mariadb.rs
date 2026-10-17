#[tokio::test]
#[ignore = "Requires WEBDB_TEST_MARIADB_CONNECTION_JSON naming a disposable MariaDB database"]
async fn browse() {
    super::super::browse(
        &dbsaur::adapters::mariadb::MariaDbAdapter,
        "WEBDB_TEST_MARIADB_CONNECTION_JSON",
        "mariadb",
        "SELECT CAST(9007199254740993 AS DECIMAL(20,0)) AS exact_value, NULL AS nullable_value",
    )
    .await;
}

#[tokio::test]
#[ignore = "Requires WEBDB_TEST_MARIADB_CONNECTION_JSON naming a disposable database with DDL permissions"]
async fn writes() {
    super::super::writes(
        &dbsaur::adapters::mariadb::MariaDbAdapter,
        "WEBDB_TEST_MARIADB_CONNECTION_JSON",
        "mariadb",
    )
    .await;
}
