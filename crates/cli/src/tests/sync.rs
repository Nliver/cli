use libpkgx::config::Config;
use libpkgx::sync;
use rusqlite::Connection;
use std::sync::Mutex;
use tempfile::TempDir;

// serialize tests that mutate PKGX_PANTRY_DIR
static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn test_config(pantry_dir: &std::path::Path, db_file: &std::path::Path) -> Config {
    Config {
        pantry_dir: pantry_dir.to_path_buf(),
        pantry_db_file: db_file.to_path_buf(),
        dist_url: "http://localhost:0".to_string(),
        pkgx_dir: pantry_dir.to_path_buf(),
    }
}

#[tokio::test]
async fn test_update_with_pantry_dir_rebuilds_db_when_projects_exists() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let pantry_dir = tmp.path().join("pantry");
    std::fs::create_dir_all(pantry_dir.join("projects")).unwrap();
    let db_file = tmp.path().join("pantry.2.db");

    let config = test_config(&pantry_dir, &db_file);
    let mut conn = Connection::open(&db_file).unwrap();

    std::env::set_var("PKGX_PANTRY_DIR", &pantry_dir);
    let result = sync::update(&config, &mut conn).await;
    std::env::remove_var("PKGX_PANTRY_DIR");

    assert!(
        result.is_ok(),
        "update should succeed when projects/ exists"
    );
}

#[tokio::test]
async fn test_update_with_pantry_dir_errors_when_projects_missing() {
    let _lock = ENV_MUTEX.lock().unwrap();
    let tmp = TempDir::new().unwrap();
    let pantry_dir = tmp.path().join("pantry");
    std::fs::create_dir_all(&pantry_dir).unwrap();
    let db_file = tmp.path().join("pantry.2.db");

    let config = test_config(&pantry_dir, &db_file);
    let mut conn = Connection::open(&db_file).unwrap();

    std::env::set_var("PKGX_PANTRY_DIR", &pantry_dir);
    let result = sync::update(&config, &mut conn).await;
    std::env::remove_var("PKGX_PANTRY_DIR");

    assert!(
        result.is_err(),
        "update should fail when projects/ is missing"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("missing projects/"),
        "error should mention missing projects/, got: {err}"
    );
}
