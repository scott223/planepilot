use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tracing::{event, Level};

pub async fn create_and_migrate_db(config: &super::Config) -> SqlitePool {
    if !Sqlite::database_exists(&config.database_url)
        .await
        .unwrap_or(false)
    {
        event!(Level::INFO, "Creating database {}", &config.database_url);
        match Sqlite::create_database(&config.database_url).await {
            Ok(_) => event!(Level::INFO, "Create db success"),
            Err(error) => panic!("error: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    let db = SqlitePool::connect(&config.database_url)
        .await
        .expect("can't connect to database");

    let migrations = std::path::Path::new(&config.migration_path);
    let migration_results = sqlx::migrate::Migrator::new(migrations)
        .await
        .unwrap()
        .run(&db)
        .await;

    match migration_results {
        Ok(_) => event!(Level::INFO, "Migration success"),
        Err(error) => {
            panic!("error: {}", error);
        }
    }

    db
}
