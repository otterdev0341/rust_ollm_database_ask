use anyhow::Error;
use sqlx::{SqlitePool, Row};
use sqlx::Column;

pub struct DbUtill;

impl DbUtill {
    // Get database schema as string
    // ("todolist", ["id", "title", "detail", "is_done", "created_at"])
    // or multiple [("todolist", ["id", "title", "detail", "is_done", "created_at"]), ("project", ["project_id", "name", "description", "deadline"])]
    pub async fn get_db_info(pool: &SqlitePool) -> Result<Vec<(String, Vec<String>)>, Error> {
        let mut tables_with_fields = Vec::new();
    
        // Step 1: Get all table names
        let tables = sqlx::query("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")
            .fetch_all(pool)
            .await?;
    
        for table in tables {
            let table_name: String = table.get("name");
    
            // Step 2: Get all columns for each table
            let pragma_query = format!("PRAGMA table_info('{}')", table_name);
            let columns = sqlx::query(&pragma_query)
                .fetch_all(pool)
                .await?;
    
            let column_names: Vec<String> = columns
                .iter()
                .map(|col| col.get::<String, _>("name"))
                .collect();
    
            tables_with_fields.push((table_name, column_names));
        }
    
        Ok(tables_with_fields)
    }

    pub async fn get_database_query(pool: &SqlitePool, sql_query: String) -> Result<String, Error> {
        // Step 1: Execute the SQL query
        let result = sqlx::query(&sql_query)
            .fetch_all(pool)
            .await;
    
        // Handle potential errors and return a custom error message
        match result {
            Ok(rows) => {
                // Step 2: Format the results as a string
                let mut formatted_result = String::new();
    
                for row in rows {
                    let columns = row.columns();
                    let mut row_data = Vec::new();
    
                    for column in columns {
                        let value: Option<String> = row.try_get(column.name()).ok(); // Use `try_get` for error handling
                        row_data.push(value.unwrap_or_else(|| "NULL".to_string()));
                    }
    
                    // Join column values and add to the result string
                    formatted_result.push_str(&row_data.join(", "));
                    formatted_result.push_str("\n");
                }
    
                Ok(formatted_result)
            },
            Err(_) => {
                // Return a custom error message if the query execution fails
                Ok("nothing to display bec can exec sql command".to_string())
            }
        }
    }

    pub fn extract_sql(response: &str) -> String {
        if let Some(start) = response.find("```sql") {
            if let Some(end) = response[start + 6..].find("```") {
                // Extract the SQL portion and trim excess whitespace
                return response[start + 6..start + 6 + end].trim().to_string();
            }
        }
    
        // Fallback: if no ```sql found, return the whole response after trimming
        response.trim().to_string()
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
