use git2::{Repository as GitRepository, DiffOptions};
use crate::models::{Commit, Repository};
use anyhow::{Result, Context};
use std::path::Path;
use std::collections::HashSet;

pub struct GitAnalyzer {
    repo: GitRepository,
    repository_info: Repository,
}

impl GitAnalyzer {
    pub fn new(repository_info: Repository) -> Result<Self> {
        let repo = GitRepository::open(&repository_info.path)
            .context(format!("Failed to open git repository at {}", repository_info.path))?;
        
        Ok(GitAnalyzer {
            repo,
            repository_info,
        })
    }

    pub fn analyze_commits(&self, since: Option<chrono::DateTime<chrono::Utc>>) -> Result<Vec<Commit>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TIME)?;

        let mut commits = Vec::new();
        let mut processed_commits = HashSet::new();

        for oid_result in revwalk {
            let oid = oid_result?;
            
            if processed_commits.contains(&oid) {
                continue;
            }
            processed_commits.insert(oid);

            let commit = self.repo.find_commit(oid)?;
            
            // Skip if commit is before the 'since' time
            if let Some(since_time) = since {
                let commit_time = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                    .unwrap_or_default();
                
                if commit_time < since_time {
                    break;
                }
            }

            // Skip merge commits to avoid double counting
            if commit.parent_count() > 1 {
                continue;
            }

            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown").to_string();
            let author_email = author.email().unwrap_or("").to_string();
            let message = commit.message().unwrap_or("").to_string();
            
            let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
                .unwrap_or_default();

            // Get current branch name if possible
            let branch = self.get_current_branch().unwrap_or_else(|_| "unknown".to_string());

            // Calculate diff stats
            let (additions, deletions, files_changed) = self.get_commit_stats(&commit)?;

            let commit_data = Commit {
                id: oid.to_string(),
                repository_id: self.repository_info.id,
                repository_name: self.repository_info.name.clone(),
                author: author_name,
                email: author_email,
                message,
                timestamp,
                additions,
                deletions,
                files_changed,
                branch: Some(branch),
            };

            commits.push(commit_data);
        }

        Ok(commits)
    }

    fn get_commit_stats(&self, commit: &git2::Commit) -> Result<(i32, i32, i32)> {
        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        diff_opts.ignore_whitespace(true);
        diff_opts.ignore_blank_lines(true);

        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            Some(&mut diff_opts),
        )?;

        let stats = diff.stats()?;
        
        Ok((
            stats.insertions() as i32,
            stats.deletions() as i32,
            stats.files_changed() as i32,
        ))
    }

    fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Ok("HEAD".to_string())
        }
    }

    pub fn is_valid_git_repo(path: &str) -> bool {
        Path::new(path).join(".git").exists() || 
        GitRepository::open(path).is_ok()
    }
}

pub fn analyze_repository(repository: Repository, since: Option<chrono::DateTime<chrono::Utc>>) -> Result<Vec<Commit>> {
    if !GitAnalyzer::is_valid_git_repo(&repository.path) {
        return Err(anyhow::anyhow!("Path is not a valid git repository: {}", repository.path));
    }

    let analyzer = GitAnalyzer::new(repository)?;
    analyzer.analyze_commits(since)
}