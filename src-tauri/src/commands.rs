use tauri::{command, AppHandle, State};
use crate::database::{self, get_db_pool};
use crate::git_analyzer::{analyze_repository, GitAnalyzer};
use crate::models::{Repository, Commit, CommitDetail, Statistics, TimeFilter};
use anyhow::Result;
use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    pub scanning: Mutex<bool>,
}

#[command]
pub async fn add_repository(app_handle: AppHandle, path: String) -> Result<Repository, String> {
    println!("Attempting to add repository: {}", path);
    
    let pool = get_db_pool(&app_handle).await.map_err(|e| {
        eprintln!("Failed to get database pool: {}", e);
        e.to_string()
    })?;
    
    // Validate that it's a git repository
    if !crate::git_analyzer::GitAnalyzer::is_valid_git_repo(&path) {
        return Err("所选路径不是有效的 Git 仓库".to_string());
    }
    
    let repository = database::add_repository(&pool, &path)
        .await
        .map_err(|e| {
            eprintln!("Failed to add repository to database: {}", e);
            if e.to_string().contains("UNIQUE constraint failed") {
                "该仓库已经添加过了".to_string()
            } else {
                format!("添加仓库失败: {}", e)
            }
        })?;
    
    println!("Successfully added repository: {} with ID: {}", path, repository.id);
    Ok(repository)
}

#[command]
pub async fn remove_repository(app_handle: AppHandle, id: i64) -> Result<(), String> {
    let pool = get_db_pool(&app_handle).await.map_err(|e| e.to_string())?;
    
    database::remove_repository(&pool, id)
        .await
        .map_err(|e| format!("删除仓库失败: {}", e))?;
    
    Ok(())
}

#[command]
pub async fn get_repositories(app_handle: AppHandle) -> Result<Vec<Repository>, String> {
    let pool = get_db_pool(&app_handle).await.map_err(|e| e.to_string())?;
    
    let repositories = database::get_repositories(&pool)
        .await
        .map_err(|e| format!("获取仓库列表失败: {}", e))?;
    
    Ok(repositories)
}

#[command]
pub async fn scan_repository(
    app_handle: AppHandle, 
    repository_id: i64,
    state: State<'_, AppState>
) -> Result<i32, String> {
    scan_repository_internal(app_handle, repository_id, state, true).await
}

#[command]
pub async fn force_scan_repository(
    app_handle: AppHandle, 
    repository_id: i64,
    state: State<'_, AppState>
) -> Result<i32, String> {
    scan_repository_internal(app_handle, repository_id, state, false).await
}

#[command]
pub async fn scan_last_24_hours(
    app_handle: AppHandle, 
    repository_id: i64,
    state: State<'_, AppState>
) -> Result<i32, String> {
    // Check if already scanning
    {
        let scanning = state.scanning.lock().unwrap();
        if *scanning {
            return Err("正在扫描中，请稍候...".to_string());
        }
    }

    // Set scanning flag
    {
        let mut scanning = state.scanning.lock().unwrap();
        *scanning = true;
    }

    let result = async {
        let pool = get_db_pool(&app_handle).await?;
        
        // Get repository info
        let repositories = database::get_repositories(&pool).await?;
        let repository = repositories
            .into_iter()
            .find(|r| r.id == repository_id)
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;
        
        // Calculate the time 24 hours ago
        let since = Some(chrono::Utc::now() - chrono::Duration::hours(24));
        
        // Analyze commits
        let analyzed_commits = analyze_repository(repository.clone(), since)?;
        let commit_count = analyzed_commits.len() as i32;
        
        // Extract commits and file changes
        let commits: Vec<Commit> = analyzed_commits.iter().map(|ac| ac.commit.clone()).collect();
        
        // Save to database
        if !commits.is_empty() {
            database::save_commits(&pool, &commits).await?;
            
            // Save file changes for each commit
            for analyzed_commit in analyzed_commits {
                database::save_file_changes(
                    &pool, 
                    &analyzed_commit.commit.id, 
                    analyzed_commit.commit.repository_id, 
                    &analyzed_commit.file_changes
                ).await?;
            }
        }
        
        // Update last scanned time
        database::update_repository_scan_time(&pool, repository_id).await?;
        
        Ok(commit_count)
    }.await;

    // Clear scanning flag
    {
        let mut scanning = state.scanning.lock().unwrap();
        *scanning = false;
    }

    result.map_err(|e: anyhow::Error| format!("扫描仓库失败: {}", e))
}

async fn scan_repository_internal(
    app_handle: AppHandle, 
    repository_id: i64,
    state: State<'_, AppState>,
    use_incremental: bool
) -> Result<i32, String> {
    // Check if already scanning
    {
        let scanning = state.scanning.lock().unwrap();
        if *scanning {
            return Err("正在扫描中，请稍候...".to_string());
        }
    }

    // Set scanning flag
    {
        let mut scanning = state.scanning.lock().unwrap();
        *scanning = true;
    }

    let result = async {
        let pool = get_db_pool(&app_handle).await?;
        
        // Get repository info
        let repositories = database::get_repositories(&pool).await?;
        let repository = repositories
            .into_iter()
            .find(|r| r.id == repository_id)
            .ok_or_else(|| anyhow::anyhow!("Repository not found"))?;
        
        // Determine since when to analyze
        // For incremental scan, only analyze new commits since last scan
        // For force scan, analyze all commits (since = None)
        let since = if use_incremental {
            repository.last_scanned
        } else {
            None
        };
        
        // Analyze commits
        let analyzed_commits = analyze_repository(repository.clone(), since)?;
        let commit_count = analyzed_commits.len() as i32;
        
        // Extract commits and file changes
        let commits: Vec<Commit> = analyzed_commits.iter().map(|ac| ac.commit.clone()).collect();
        
        // Save to database
        if !commits.is_empty() {
            database::save_commits(&pool, &commits).await?;
            
            // Save file changes for each commit
            for analyzed_commit in analyzed_commits {
                database::save_file_changes(
                    &pool, 
                    &analyzed_commit.commit.id, 
                    analyzed_commit.commit.repository_id, 
                    &analyzed_commit.file_changes
                ).await?;
            }
        }
        
        // Update last scanned time (only for incremental scan)
        if use_incremental {
            database::update_repository_scan_time(&pool, repository_id).await?;
        }
        
        Ok(commit_count)
    }.await;

    // Clear scanning flag
    {
        let mut scanning = state.scanning.lock().unwrap();
        *scanning = false;
    }

    result.map_err(|e: anyhow::Error| format!("扫描仓库失败: {}", e))
}

#[command]
pub async fn get_statistics(
    app_handle: AppHandle,
    start_date: Option<String>,
    end_date: Option<String>,
    author: Option<String>,
    exclude_authors: Option<Vec<String>>,
    repository_id: Option<i64>
) -> Result<Statistics, String> {
    let pool = get_db_pool(&app_handle).await.map_err(|e| e.to_string())?;
    
    let filter = TimeFilter {
        start_date: start_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        end_date: end_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        author,
        exclude_authors,
        repository_id,
    };
    
    let statistics = database::get_statistics(&pool, &filter)
        .await
        .map_err(|e| format!("获取统计数据失败: {}", e))?;
    
    Ok(statistics)
}

#[command]
pub async fn get_commit_timeline(
    app_handle: AppHandle,
    start_date: Option<String>,
    end_date: Option<String>,
    author: Option<String>,
    exclude_authors: Option<Vec<String>>,
    repository_id: Option<i64>
) -> Result<Vec<Commit>, String> {
    let pool = get_db_pool(&app_handle).await.map_err(|e| e.to_string())?;
    
    let filter = TimeFilter {
        start_date: start_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        end_date: end_date.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc)),
        author,
        exclude_authors,
        repository_id,
    };
    
    let commits = database::get_commit_timeline(&pool, &filter)
        .await
        .map_err(|e| format!("获取提交时间线失败: {}", e))?;
    
    Ok(commits)
}

#[command]
pub async fn get_commit_detail(
    app_handle: AppHandle,
    repository_id: i64,
    commit_id: String
) -> Result<CommitDetail, String> {
    let pool = get_db_pool(&app_handle).await.map_err(|e| e.to_string())?;
    
    // Get repository info
    let repositories = database::get_repositories(&pool)
        .await
        .map_err(|e| format!("获取仓库信息失败: {}", e))?;
        
    let repository = repositories
        .into_iter()
        .find(|r| r.id == repository_id)
        .ok_or_else(|| "仓库未找到".to_string())?;
    
    // Create GitAnalyzer and get commit detail
    let analyzer = GitAnalyzer::new(repository)
        .map_err(|e| format!("无法打开仓库: {}", e))?;
        
    let commit_detail = analyzer.get_commit_detail(&commit_id)
        .map_err(|e| format!("获取提交详情失败: {}", e))?;
    
    Ok(commit_detail)
}