use anyhow::{Error, Ok};
use async_trait::async_trait;
use ollama_rs::Ollama;
use sqlx::SqlitePool;

use crate::{configuration::{db_config::DbConfig, ollama_config::OllamaConfig}, trait_req_impl::chain::Chain};

pub struct TextToSqlChain{
    client: Ollama,
    db: SqlitePool
}


#[async_trait]
impl Chain for TextToSqlChain {
    async fn initialize() -> Result<Box<dyn Chain + Send>, Error>
    where 
        Self: Sized
    {
        let db_config = DbConfig::inject_from_env();
        let ollama_config = OllamaConfig::inject_from_env();
        let pool = SqlitePool::connect(&db_config.url).await?;
        Ok(
            Box::new(TextToSqlChain{
                client: Ollama::new(ollama_config.url, ollama_config.port),
                db: pool
            })
        )
    }
            
    async fn run(&self, input: String) -> Result<String, Error> {
        unimplemented!()
        
    }
}