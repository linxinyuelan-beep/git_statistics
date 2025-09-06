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
        "#
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS file_changes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            commit_id TEXT NOT NULL,
            repository_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            additions INTEGER NOT NULL DEFAULT 0,
            deletions INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (commit_id, repository_id) REFERENCES commits (id, repository_id) ON DELETE CASCADE
        )
        "#
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

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_changes_path ON file_changes(file_path)")
        .execute(&pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_file_changes_commit ON file_changes(commit_id)")
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

pub async fn save_file_changes(pool: &SqlitePool, commit_id: &str, repository_id: i64, file_changes: &[crate::git_analyzer::FileChange]) -> Result<()> {
    let mut tx = pool.begin().await?;
    
    for file_change in file_changes {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO file_changes 
            (commit_id, repository_id, file_path, additions, deletions)
            VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(commit_id)
        .bind(repository_id)
        .bind(&file_change.path)
        .bind(file_change.additions)
        .bind(file_change.deletions)
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
    
    if let Some(exclude_authors) = &filter.exclude_authors {
        if !exclude_authors.is_empty() {
            let placeholders: Vec<String> = exclude_authors.iter().map(|_| "?".to_string()).collect();
            query.push_str(&format!(" AND author NOT IN ({})", placeholders.join(",")));
            for author in exclude_authors {
                params.push(author.clone());
            }
        }
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
    
    if let Some(exclude_authors) = &filter.exclude_authors {
        if !exclude_authors.is_empty() {
            let placeholders: Vec<String> = exclude_authors.iter().map(|_| "?".to_string()).collect();
            base_query.push_str(&format!(" AND author NOT IN ({})", placeholders.join(",")));
            for author in exclude_authors {
                params.push(author.clone());
            }
        }
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

    // Get hourly commit distribution for heatmap (hour x day of week)
    let hourly_dist_query = format!(
        "SELECT strftime('%H', timestamp, 'localtime') as hour,
         strftime('%w', timestamp, 'localtime') as day_of_week,
         COUNT(*) as commits
         {} GROUP BY hour, day_of_week ORDER BY hour, day_of_week",
        base_query
    );
    
    let mut query_builder = sqlx::query(&hourly_dist_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let hourly_dist_rows = query_builder.fetch_all(pool).await?;
    let hourly_commit_distribution: Vec<HourlyCommitDistribution> = hourly_dist_rows
        .into_iter()
        .map(|row| HourlyCommitDistribution {
            hour: row.get::<String, _>("hour").parse().unwrap_or(0),
            day_of_week: row.get::<String, _>("day_of_week").parse().unwrap_or(0),
            commits: row.get("commits"),
        })
        .collect();

    // Get author activity trends (daily)
    let author_trend_query = format!(
        "SELECT author,
         DATE(timestamp) as period,
         COUNT(*) as commits,
         SUM(additions) as additions,
         SUM(deletions) as deletions
         {} GROUP BY author, period ORDER BY period, commits DESC",
        base_query
    );
    
    let mut query_builder = sqlx::query(&author_trend_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let author_trend_rows = query_builder.fetch_all(pool).await?;
    let author_activity_trends: Vec<AuthorActivityTrend> = author_trend_rows
        .into_iter()
        .map(|row| AuthorActivityTrend {
            author: row.get("author"),
            period: row.get("period"),
            commits: row.get("commits"),
            additions: row.get("additions"),
            deletions: row.get("deletions"),
        })
        .collect();

    // Get commit frequency distribution (commits per day)
    let freq_dist_query = format!(
        "SELECT DATE(timestamp) as date,
         COUNT(*) as commit_count
         {} GROUP BY date ORDER BY date",
        base_query
    );
    
    let mut query_builder = sqlx::query(&freq_dist_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let freq_dist_rows = query_builder.fetch_all(pool).await?;
    let commit_frequency_distribution: Vec<CommitFrequencyDistribution> = freq_dist_rows
        .into_iter()
        .map(|row| CommitFrequencyDistribution {
            date: row.get("date"),
            commit_count: row.get("commit_count"),
        })
        .collect();

    // Get commit size distribution
    let size_dist_query = format!(
        "SELECT 
         CASE 
           WHEN (additions + deletions) <= 10 THEN 'small'
           WHEN (additions + deletions) <= 100 THEN 'medium'
           WHEN (additions + deletions) <= 500 THEN 'large'
           ELSE 'huge'
         END as size_range,
         COUNT(*) as count
         {} GROUP BY size_range ORDER BY 
         CASE size_range 
           WHEN 'small' THEN 1
           WHEN 'medium' THEN 2
           WHEN 'large' THEN 3
           WHEN 'huge' THEN 4
         END",
        base_query
    );
    
    let mut query_builder = sqlx::query(&size_dist_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let size_dist_rows = query_builder.fetch_all(pool).await?;
    let commit_size_distribution: Vec<CommitSizeDistribution> = size_dist_rows
        .into_iter()
        .map(|row| {
            let size_range: String = row.get("size_range");
            let (min_lines, max_lines) = match size_range.as_str() {
                "small" => (0, 10),
                "medium" => (11, 100),
                "large" => (101, 500),
                "huge" => (501, i32::MAX),
                _ => (0, 0),
            };
            CommitSizeDistribution {
                size_range,
                count: row.get("count"),
                min_lines,
                max_lines,
            }
        })
        .collect();

    // Get efficiency trends (additions / (additions + deletions))
    let efficiency_query = format!(
        "SELECT DATE(timestamp) as date,
         SUM(additions) as total_additions,
         SUM(deletions) as total_deletions,
         (SUM(additions) + SUM(deletions)) as total_changes
         {} GROUP BY date ORDER BY date",
        base_query
    );
    
    let mut query_builder = sqlx::query(&efficiency_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let efficiency_rows = query_builder.fetch_all(pool).await?;
    let efficiency_trends: Vec<EfficiencyTrend> = efficiency_rows
        .into_iter()
        .map(|row| {
            let additions: i32 = row.get("total_additions");
            let deletions: i32 = row.get("total_deletions");
            let total = additions + deletions;
            let efficiency_ratio = if total > 0 {
                additions as f64 / total as f64
            } else {
                0.5
            };
            EfficiencyTrend {
                date: row.get("date"),
                efficiency_ratio,
                total_changes: row.get("total_changes"),
            }
        })
        .collect();

    // Get hot files (most frequently changed files)
    // We now have actual file-level data stored in the database
    let hot_files_query = format!(
        "SELECT fc.file_path,
         COUNT(*) as change_count,
         SUM(fc.additions) as total_additions,
         SUM(fc.deletions) as total_deletions,
         MAX(c.timestamp) as last_modified
         FROM file_changes fc
         JOIN commits c ON fc.commit_id = c.id AND fc.repository_id = c.repository_id
         WHERE 1=1 {}
         GROUP BY fc.file_path 
         ORDER BY change_count DESC 
         LIMIT 20",
        {
            let mut conditions = String::new();
            if let Some(start_date) = &filter.start_date {
                conditions.push_str(" AND c.timestamp >= ?");
            }
            if let Some(end_date) = &filter.end_date {
                conditions.push_str(" AND c.timestamp <= ?");
            }
            if let Some(author) = &filter.author {
                conditions.push_str(" AND c.author = ?");
            }
            if let Some(exclude_authors) = &filter.exclude_authors {
                if !exclude_authors.is_empty() {
                    let placeholders: Vec<String> = exclude_authors.iter().map(|_| "?".to_string()).collect();
                    conditions.push_str(&format!(" AND c.author NOT IN ({})", placeholders.join(",")));
                }
            }
            if let Some(repository_id) = filter.repository_id {
                conditions.push_str(" AND c.repository_id = ?");
            }
            conditions
        }
    );
    
    let mut query_builder = sqlx::query(&hot_files_query);
    // Bind parameters for the file changes query
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    // Bind additional parameters for exclude_authors if needed
    if let Some(exclude_authors) = &filter.exclude_authors {
        for author in exclude_authors {
            query_builder = query_builder.bind(author);
        }
    }
    
    let hot_files_rows = query_builder.fetch_all(pool).await?;
    let hot_files: Vec<HotFile> = hot_files_rows
        .into_iter()
        .map(|row| HotFile {
            file_path: row.get("file_path"),
            change_count: row.get("change_count"),
            total_additions: row.get("total_additions"),
            total_deletions: row.get("total_deletions"),
            last_modified: row.get::<chrono::DateTime<chrono::Utc>, _>("last_modified").to_rfc3339(),
        })
        .collect();

    // Get commit message words (basic word frequency analysis)
    let message_query = format!(
        "SELECT message {} ORDER BY timestamp DESC LIMIT 1000",
        base_query
    );
    
    let mut query_builder = sqlx::query(&message_query);
    for param in &params {
        query_builder = query_builder.bind(param);
    }
    
    let message_rows = query_builder.fetch_all(pool).await?;
    let mut word_counts = std::collections::HashMap::new();
    
    // Process commit messages to extract words
    for row in message_rows {
        let message: String = row.get("message");
        let words: Vec<&str> = message
            .split_whitespace()
            .filter(|word| word.len() > 2 && !word.starts_with('#'))
            .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|word| word.len() > 2)
            .collect();
        
        for word in words {
            let word = word.to_lowercase();
            // Skip common words
            if !matches!(word.as_str(), "the" | "and" | "for" | "are" | "but" | "not" | "you" | "all" | "can" | "her" | "was" | "one" | "our" | "out" | "day" | "get" | "use" | "man" | "new" | "now" | "way" | "may" | "say" | "each" | "which" | "their" | "time" | "will" | "about" | "if" | "up" | "out" | "many" | "then" | "them" | "these" | "so" | "some" | "her" | "would" | "make" | "like" | "into" | "him" | "has" | "two" | "more" | "very" | "what" | "know" | "just" | "first" | "could" | "any" | "my" | "than" | "much" | "your" | "how" | "said" | "each" | "she" | "which" | "their" | "his" | "been" | "have" | "there" | "we" | "what" | "were" | "they" | "who" | "oil" | "its" | "now" | "find" | "long" | "down" | "day" | "did" | "get" | "come" | "made" | "may" | "part") {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }
    }
    
    // Convert to sorted list
    let mut commit_message_words: Vec<CommitMessageWord> = word_counts
        .into_iter()
        .filter(|(_, count)| *count >= 2) // Only include words that appear at least twice
        .map(|(word, count)| CommitMessageWord {
            word,
            count,
            weight: (count as f64).log2() + 1.0, // Log scale for better visualization
        })
        .collect();
    
    commit_message_words.sort_by(|a, b| b.count.cmp(&a.count));
    commit_message_words.truncate(50); // Limit to top 50 words

    Ok(Statistics {
        hourly,
        daily,
        weekly,
        total_commits,
        total_additions,
        total_deletions,
        authors,
        repositories,
        hourly_commit_distribution,
        author_activity_trends,
        commit_frequency_distribution,
        commit_size_distribution,
        efficiency_trends,
        hot_files,
        commit_message_words,
    })
}