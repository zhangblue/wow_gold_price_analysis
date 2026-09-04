use gold_price_backend::config::{load_database_url, ReleasePaths};
use tempfile::tempdir;

#[test]
fn loads_database_url_from_the_release_config_directory() {
    let release = tempdir().unwrap();
    std::fs::create_dir(release.path().join("config")).unwrap();
    std::fs::write(
        release.path().join("config/.env"),
        "DATABASE_URL=postgres://example/db\n",
    )
    .unwrap();

    assert_eq!(
        load_database_url(&ReleasePaths::from_release_dir(release.path())).unwrap(),
        "postgres://example/db",
    );
}

#[test]
fn reports_a_missing_database_url() {
    let release = tempdir().unwrap();
    std::fs::create_dir(release.path().join("config")).unwrap();
    std::fs::write(release.path().join("config/.env"), "OTHER=value\n").unwrap();

    assert!(
        load_database_url(&ReleasePaths::from_release_dir(release.path()))
            .unwrap_err()
            .to_string()
            .contains("DATABASE_URL")
    );
}
