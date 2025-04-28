use anyhow::Error;
use async_trait::async_trait;
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use sqlx::SqlitePool;
use std::result::Result::Ok;
use crate::{configuration::{db_config::DbConfig, model_config::ModelSelect, ollama_config::OllamaConfig}, trait_req_impl::chain::Chain};

use super::db_utill::DbUtill;

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
        // send prompt with db schma to get back sql command
        let prompt = match self.construct_prompt(input.clone()).await {
            Ok(data) => data,
            Err(_) => panic!("Fail to init prompt in run method")
        };
        let request = GenerationRequest::new(
            ModelSelect::NplOperate.as_str(), 
            prompt);

        // get sql command then execute to get result and send back to llm again
        let raw_response = self.client.generate(request).await.unwrap().response;
        let trim_sql = DbUtill::extract_sql(&raw_response);
        println!("RAW SQL is : {}", trim_sql);
        let database_information = DbUtill::get_database_query(&self.db, trim_sql).await.unwrap();
        println!("Database retrive: {}", database_information);
        let second_prompt = self.prepare_prompt(input, database_information).await.unwrap();
        let second_request = GenerationRequest::new(
            ModelSelect::NplOperate.as_str(),
            second_prompt
        );
        let final_response = self.client.generate(second_request).await.unwrap().response;
        Ok(final_response)
        
    }
}

impl TextToSqlChain {
    async fn construct_prompt(&self, input: String) -> Result<String, Error> {
        let db_schema = DbUtill::get_db_info(&self.db).await
            .expect("Failed to get schema");
    
        // Format schema for better readability
        let prompt = format!(
            "You are a database expert.
    
    Database Schema:
    {}
    
    Instructions:
    - Generate ONE correct SQL query for SQLite that answers the given user question.
    - Only output the SQL command.
    - No explanations, no examples, no prefixes (such as 'Example:', 'SQL:', 'Response:', 'Result:').
    - No formatting like markdown (no ```sql blocks).
    - Output ONLY the SQL query — no extra text.
    
    User Question:
    {}
    
    Remember: ONLY output a valid SQL command.",
            db_schema,
            input.trim()
        );
    
        Ok(prompt)
    }
    
    


    async fn prepare_prompt(&self, question: String, context: String) -> Result<String, Error> {
        let context = format!("base on provid data {}. please answer the question {} with natural language",context, question);
        Ok(context)

    }
}