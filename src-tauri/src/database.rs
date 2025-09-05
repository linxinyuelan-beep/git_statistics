use sqlx::{SqlitePool, Row};
use tauri::{AppHandle, api::path};
use crate::models::*;
use anyhow::Result;

pub async fn init_database(app_handle: &AppHandle) -> Result<SqlitePool> {
    let app_dir = path::app_data_dir(&app_handle.config())
        .ok_or_else(|| anyhow::anyhow!("Failed to get app data dir"))?;
    
    // Ensure the directory exists
    if let Err(e) = tokio::fs::create_dir_all(&app_dir).await {
        return Err(anyhow::anyhow!("Failed to create app data directory: {}", e));
    }
    
    let db_path = app_dir.join("git_stats.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    
    println!("Initializing database at: {}", db_url);
    let pool = SqlitePool::connect(&db_url).await?;
    
    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repositories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            last_scanned DATETIME
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS commits (
            id TEXT NOT NULL,
            repository_id INTEGER NOT NULL,
            repository_name TEXT NOT NULL,
            author TEXT NOT NULL,
            email TEXT NOT NULL,
            message TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            additions INTEGER NOT NULL DEFAULT 0,
            deletions INTEGER NOT NULL DEFAULT 0,
            files_changed INTEGER NOT NULL DEFAULT 0,
            branch TEXT,
            PRIMARY KEY (id, repository_id),
            FOREIGN KEY (repository_id) REFERENCES repositories (id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Create indexes for performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_commits_timestamp ON commits(timestamp)")
        .execute(&pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_commits_author ON commits(author)")
        .execute(&pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_commits_repository ON commits(repository_id)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn get_db_pool(app_handle: &AppHandle) -> Result<SqlitePool> {
    let app_dir = path::app_data_dir(&app_handle.config())
        .ok_or_else(|| anyhow::anyhow!("Failed to get app data dir"))?;
    
    // Ensure the directory exists
    tokio::fs::create_dir_all(&app_dir).await?;
    
    let db_path = app_dir.join("git_stats.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    
    Ok(SqlitePool::connect(&db_url).await?)
}

pub async fn add_repository(pool: &SqlitePool, path: &str) -> Result<Repository> {
    let name = std::path::Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let result = sqlx::query(
        "INSERT INTO repositories (path, name) VALUES (?, ?) RETURNING id, path, name, last_scanned"
    )
    .bind(path)
    .bind(&name)
    .fetch_one(pool)
    .await?;

    Ok(Repository {
        id: result.get("id"),
        path: result.get("path"),
        name: result.get("name"),
        last_scanned: result.get("last_scanned"),
    })
}

pub async fn remove_repository(pool: &SqlitePool, id: i64) -> Result<()> {
    sqlx::query("DELETE FROM repositories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_repositories(pool: &SqlitePool) -> Result<Vec<Repository>> {
    let repositories = sqlx::query_as::<_, Repository>(
        "SELECT id, path, name, last_scanned FROM repositories ORDER BY name"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(repositories)
}

pub async fn update_repository_scan_time(pool: &SqlitePool, id: i64) -> Result<()> {
    sqlx::query("UPDATE repositories SET last_scanned = ? WHERE id = ?")
        .bind(chrono::Utc::now())
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn save_commits(pool: &SqlitePool, commits: &[Commit]) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    for commit in commits {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO commits 
            (id, repository_id, repository_name, author, email, message, timestamp, additions, deletions, files_changed, branch)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&commit.id)
        .bind(commit.repository_id)
        .bind(&commit.repository_name)
        .bind(&commit.author)
        .bind(&commit.email)
        .bind(&commit.message)
        .bind(commit.timestamp)
        .bind(commit.additions)
        .bind(commit.deletions)
        .bind(commit.files_changed)
        .bind(&commit.branch)
        .execute(&mut *tx)
        .await?;
    }
    
    tx.commit().await?;
    Ok(())
}

pub async fn get_commit_timeline(pool: &SqlitePool, filter: &TimeFilter) -> Result<Vec<Commit>> {
    let mut query = "SELECT * FROM commits WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();
    
    if let Some(start_date) = &filter.start_date {
        query.push_str(" AND timestamp >= ?");
        params.push(start_date.to_rfc3339());
    }
    
    if let Some(end_date) = &filter.end_date {
        query.push_str(" AND timestamp <= ?");
        params.push(end_date.to_rfc3339());
    }
    
    if let Some(author) = &filter.author {
        query.push_str(" AND author = ?");
        params.push(author.clone());
    }
    
    if let Some(repository_id) = filter.repository_id {
        query.push_str(" AND repository_id = ?");
        params.push(repository_id.to_string());
    }
    
    query.push_str(" ORDER BY timestamp DESC LIMIT 1000");
    
    let mut query_builder = sqlx::query_as::<_, Commit>(&query);
    
    for param in params {
        query_builder = query_builder.bind(param);
    }
    
    let commits = query_builder.fetch_all(pool).await?;
    Ok(commits)
}

pub async fn get_statistics(pool: &SqlitePool, filter: &TimeFilter) -> Result<Statistics> {
    let mut base_query = "FROM commits WHERE 1=1".to_string();
    let mut params: Vec<String> = Vec::new();
    
    if let Some(start_date) = &filter.start_date {
        base_query.push_str(" AND timestamp >= ?");
        params.push(start_date.to_rfc3339());
    }
    
    if let Some(end_date) = &filter.end_date {
        base_query.push_str(" AND timestamp <= ?");
        params.push(end_date.to_rfc3339());
    }
    
    if let Some(author) = &filter.author {
        base_query.push_str(" AND author = ?");
        params.push(author.clone());
    }
    
    if let Some(repository_id) = filter.repository_id {
        base_query.push_str(" AND repository_id = ?");
        params.push(repository_id.to_string());
    }

    // Get hourly stats (convert UTC to local time for proper hour grouping)
    let hourly_query = format!(
        "SELECT strftime('%H', timestamp, 'localtime') as hour, 
         SUM(additions) as additions, 
         SUM(deletions) as deletions, 
         COUNT(*) as commits 
         {} GROUP BY hour ORDER BY hour",
        base_query
    );
    
    let mut query_builder = sqlx::query(&hourly_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let hourly_rows = query_builder.fetch_all(pool).await?;
    let hourly: Vec<HourlyStats> = hourly_rows
        .into_iter()
        .map(|row| HourlyStats {
            hour: row.get::<String, _>("hour").parse().unwrap_or(0),
            additions: row.get("additions"),
            deletions: row.get("deletions"),
            commits: row.get("commits"),
        })
        .collect();

    // Get daily stats
    let daily_query = format!(
        "SELECT DATE(timestamp) as date, 
         SUM(additions) as additions, 
         SUM(deletions) as deletions, 
         COUNT(*) as commits 
         {} GROUP BY date ORDER BY date DESC LIMIT 30",
        base_query
    );
    
    let mut query_builder = sqlx::query(&daily_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let daily_rows = query_builder.fetch_all(pool).await?;
    let daily: Vec<DailyStats> = daily_rows
        .into_iter()
        .map(|row| DailyStats {
            date: row.get("date"),
            additions: row.get("additions"),
            deletions: row.get("deletions"),
            commits: row.get("commits"),
        })
        .collect();

    // Get weekly stats (convert UTC to local time for proper weekday grouping)
    let weekly_query = format!(
        "SELECT strftime('%w', timestamp, 'localtime') as weekday, 
         SUM(additions) as additions, 
         SUM(deletions) as deletions, 
         COUNT(*) as commits 
         {} GROUP BY weekday ORDER BY weekday",
        base_query
    );
    
    let mut query_builder = sqlx::query(&weekly_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let weekly_rows = query_builder.fetch_all(pool).await?;
    let weekly: Vec<WeeklyStats> = weekly_rows
        .into_iter()
        .map(|row| WeeklyStats {
            weekday: row.get::<String, _>("weekday").parse().unwrap_or(0),
            additions: row.get("additions"),
            deletions: row.get("deletions"),
            commits: row.get("commits"),
        })
        .collect();

    // Get total stats
    let total_query = format!(
        "SELECT SUM(additions) as total_additions, 
         SUM(deletions) as total_deletions, 
         COUNT(*) as total_commits 
         {}",
        base_query
    );
    
    let mut query_builder = sqlx::query(&total_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let total_row = query_builder.fetch_one(pool).await?;
    let total_commits: i32 = total_row.get("total_commits");
    let total_additions: i32 = total_row.get("total_additions");
    let total_deletions: i32 = total_row.get("total_deletions");

    // Get author stats
    let author_query = format!(
        "SELECT author, 
         SUM(additions) as additions, 
         SUM(deletions) as deletions, 
         COUNT(*) as commits 
         {} GROUP BY author ORDER BY (additions + deletions) DESC",
        base_query
    );
    
    let mut query_builder = sqlx::query(&author_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let author_rows = query_builder.fetch_all(pool).await?;
    let mut authors = std::collections::HashMap::new();
    for row in author_rows {
        let author: String = row.get("author");
        let stats = AuthorStats {
            additions: row.get("additions"),
            deletions: row.get("deletions"),
            commits: row.get("commits"),
        };
        authors.insert(author, stats);
    }

    // Get repository stats
    let repo_query = format!(
        "SELECT repository_name, 
         SUM(additions) as additions, 
         SUM(deletions) as deletions, 
         COUNT(*) as commits 
         {} GROUP BY repository_name ORDER BY (additions + deletions) DESC",
        base_query
    );
    
    let mut query_builder = sqlx::query(&repo_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let repo_rows = query_builder.fetch_all(pool).await?;
    let mut repositories = std::collections::HashMap::new();
    for row in repo_rows {
        let repo_name: String = row.get("repository_name");
        let stats = RepositoryStats {
            additions: row.get("additions"),
            deletions: row.get("deletions"),
            commits: row.get("commits"),
        };
        repositories.insert(repo_name, stats);
    }

    Ok(Statistics {
        hourly,
        daily,
        weekly,
        total_commits,
        total_additions,
        total_deletions,
        authors,
        repositories,
    })
}