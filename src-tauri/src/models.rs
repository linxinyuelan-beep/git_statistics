use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub last_scanned: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Commit {
    pub id: String,
    pub repository_id: i64,
    pub repository_name: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additions: i32,
    pub deletions: i32,
    pub files_changed: i32,
    pub branch: Option<String>,
}

// New struct for file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub additions: i32,
    pub deletions: i32,
    pub diff: String,
}

// New struct for commit details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetail {
    pub id: String,
    pub repository_id: i64,
    pub repository_name: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additions: i32,
    pub deletions: i32,
    pub files_changed: i32,
    pub branch: Option<String>,
    pub file_changes: Vec<FileChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: i32,
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeeklyStats {
    pub weekday: i32, // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorStats {
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub additions: i32,
    pub deletions: i32,
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HourlyCommitDistribution {
    pub hour: i32,
    pub day_of_week: i32, // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
    pub commits: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorActivityTrend {
    pub author: String,
    pub period: String, // Format: "YYYY-MM" for monthly
    pub commits: i32,
    pub additions: i32,
    pub deletions: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitFrequencyDistribution {
    pub date: String,
    pub commit_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitSizeDistribution {
    pub size_range: String, // "small", "medium", "large", "huge"
    pub count: i32,
    pub min_lines: i32,
    pub max_lines: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EfficiencyTrend {
    pub date: String,
    pub efficiency_ratio: f64, // additions / (additions + deletions)
    pub total_changes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HotFile {
    pub file_path: String,
    pub change_count: i32,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub last_modified: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommitMessageWord {
    pub word: String,
    pub count: i32,
    pub weight: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub hourly: Vec<HourlyStats>,
    pub daily: Vec<DailyStats>,
    pub weekly: Vec<WeeklyStats>,
    pub total_commits: i32,
    pub total_additions: i32,
    pub total_deletions: i32,
    pub authors: std::collections::HashMap<String, AuthorStats>,
    pub repositories: std::collections::HashMap<String, RepositoryStats>,
    // New fields for additional charts
    pub hourly_commit_distribution: Vec<HourlyCommitDistribution>,
    pub author_activity_trends: Vec<AuthorActivityTrend>,
    pub commit_frequency_distribution: Vec<CommitFrequencyDistribution>,
    // New charts data
    pub commit_size_distribution: Vec<CommitSizeDistribution>,
    pub efficiency_trends: Vec<EfficiencyTrend>,
    pub hot_files: Vec<HotFile>,
    pub commit_message_words: Vec<CommitMessageWord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeFilter {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub author: Option<String>,
    pub exclude_authors: Option<Vec<String>>,
    pub repository_id: Option<i64>,
}