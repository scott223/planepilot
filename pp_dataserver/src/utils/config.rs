use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub migration_path: String,
    pub data_frame_duration: i32, // how many minutes the standard api request for data is
    pub data_frame_offset: i32,   //how many minutes offset since now (back in time)
}

impl Default for Config {
    fn default() -> Self {
        let database_url: String =
            env::var("DATABASE_URL").unwrap_or("./pp_dataserver/sqlite.db".to_string());
        let migration_path: String =
            env::var("MIGRATION_PATH").unwrap_or("./pp_dataserver/migrations".to_string());
        Config {
            database_url,
            migration_path,
            data_frame_duration: 15,
            data_frame_offset: 0,
        }
    }
}
