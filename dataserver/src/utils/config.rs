use std::env;

pub struct Config {
    pub database_url: String,
    pub migration_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let database_url: String =
            env::var("DATABASE_URL").expect("cannot find database url in .env");
        let migration_path: String =
            env::var("MIGRATION_PATH").expect("cannot find migration path in .env");
        Config {
            database_url,
            migration_path,
        }
    }
}
