use anyhow::Error;
use sqlx::{SqlitePool, Row};
use sqlx::Column;

pub struct DbUtill;

impl DbUtill {
    // Get database schema as string
    // ("todolist", ["id", "title", "detail", "is_done", "created_at"])
    // or multiple [("todolist", ["id", "title", "detail", "is_done", "created_at"]), ("project", ["project_id", "name", "description", "deadline"])]
    pub async fn get_db_info(pool: &SqlitePool) -> Result<String, Error> {
        let mut schema_description = String::new();
    
        // Step 1: Get all table names
        let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
            .fetch_all(pool)
            .await?;
    
        for table in tables {
            let table_name: String = table.get("name");
    
            schema_description.push_str(&format!("Table \"{}\":\n", table_name));
    
            // Step 2: Get all columns for each table (including name and type)
            let pragma_query = format!("PRAGMA table_info('{}')", table_name);
            let columns = sqlx::query(&pragma_query)
                .fetch_all(pool)
                .await?;
    
            for col in columns {
                let column_name: String = col.get("name");
                let column_type: String = col.get("type");
                schema_description.push_str(&format!("- {} ({})\n", column_name, column_type));
            }
    
            schema_description.push('\n');
        }
    
        Ok(schema_description)
    }

    pub async fn get_database_query(pool: &SqlitePool, sql_query: String) -> Result<String, Error> {
        // Step 1: Execute the SQL query
        let result = sqlx::query(&sql_query)
            .fetch_all(pool)  // Use fetch_all to handle multiple rows if needed
            .await;
    
        // Handle potential errors and return a custom error message
        match result {
            Ok(rows) => {
                // Step 2: Format the results dynamically
                let mut formatted_result = String::new();
    
                for row in rows {
                    let mut row_data = Vec::new();
    
                    for (i, column) in row.columns().iter().enumerate() {
                        // Dynamically handle any column in the row
                        let value: Option<String> = row.try_get(i).ok(); // Get the column value
                        let value_str = value.unwrap_or_else(|| "NULL".to_string()); // Handle missing values
    
                        println!("Column: {}, Value: {}", column.name(), value_str); // Log values
                        row_data.push(value_str);
                    }
    
                    // Join column values and add to the result string
                    formatted_result.push_str(&row_data.join(", "));
                    formatted_result.push_str("\n");
                }
    
                Ok(formatted_result)
            },
            Err(e) => {
                Err(anyhow::anyhow!("Database query failed: {}", e).into()) // Improved error handling
            }
        }
    }

    pub fn extract_sql(response: &str) -> String {
        let trimmed = response.trim();
    
        let sql_start = trimmed.find("SELECT")
            .or_else(|| trimmed.find("INSERT"))
            .or_else(|| trimmed.find("UPDATE"))
            .or_else(|| trimmed.find("DELETE"));
    
        if let Some(start) = sql_start {
            let after_start = &trimmed[start..];
            if let Some(response_pos) = after_start.find("Response :") {
                after_start[..response_pos]
                    .trim()
                    .trim_end_matches(';')
                    .to_string()
            } else {
                after_start
                    .trim()
                    .trim_end_matches(';')
                    .to_string()
            }
        } else {
            trimmed.to_string()
        }
    }
    

}

#[cfg(test)]
pub mod test {
    use super::*;
    use sqlx::SqlitePool;
    use anyhow::Error;

    #[tokio::test]
    async fn test_get_database_query() -> Result<(), Error> {
        let pool = SqlitePool::connect(":memory:").await?;

        // Create a sample table
        sqlx::query(
            "CREATE TABLE todolist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                detail TEXT,
                is_done BOOLEAN NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );"
        )
        .execute(&pool)
        .await?;

        // Insert sample data
        sqlx::query("INSERT INTO todolist (title, detail) VALUES ('Task 1', 'Detail 1')")
            .execute(&pool)
            .await?;

        sqlx::query("INSERT INTO todolist (title, detail) VALUES ('Task 2', 'Detail 2')")
            .execute(&pool)
            .await?;

        // Define an SQL query to retrieve data
        let sql_query = "SELECT id, title, detail, is_done, created_at FROM todolist;".to_string();

        // Get the result of the query as a string
        let query_result = DbUtill::get_database_query(&pool, sql_query).await?;
        print!("{:?}",query_result);
        // // Validate the result
        // assert!(query_result.contains("Task 1"));
        // assert!(query_result.contains("Task 2"));
        // assert!(query_result.contains("title"));

        Ok(())
    }
}
