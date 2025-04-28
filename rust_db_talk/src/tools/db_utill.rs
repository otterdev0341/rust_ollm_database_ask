use anyhow::Error;
use sqlx::SqlitePool;
use sqlx::Row;


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


#[cfg(test)]
pub mod test {
    use anyhow::Error;
    use sqlx::SqlitePool;
    use super::get_db_info;

    #[tokio::test]
    async fn test_get_table_name() -> Result<(), Error> {
        let pool = SqlitePool::connect(":memory:").await?;

        // Create first table
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

        // Create second table
        sqlx::query(
            "CREATE TABLE project (
                project_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                deadline DATE
            );"
        )
        .execute(&pool)
        .await?;

        // Call your function
        let mut tables = get_db_info(&pool).await?;
        println!("{:?}", tables);
        // Sort the result to make sure the order is predictable
        tables.sort_by(|a, b| a.0.cmp(&b.0));

        // Check the number of tables
        assert_eq!(tables.len(), 2);

        // Check first table
        assert_eq!(tables[0].0, "project");
        assert_eq!(
            tables[0].1,
            vec!["project_id", "name", "description", "deadline"]
        );

        // Check second table
        assert_eq!(tables[1].0, "todolist");
        assert_eq!(
            tables[1].1,
            vec!["id", "title", "detail", "is_done", "created_at"]
        );

        Ok(())
    }
}