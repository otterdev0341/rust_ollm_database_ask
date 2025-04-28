use std::env;

pub struct DbConfig{
    pub url: String
}



impl DbConfig{
    pub fn inject_from_env() -> Self {
        dotenv::dotenv().ok();
        let database_url = match env::var("DATABASE_URL") {
            Ok(data) => data,
            Err(_) => "".to_string()
        };
        Self { url: database_url }

    }
}