export interface Repository {
  id: number;
  path: string;
  name: string;
  last_scanned?: string;
}

export interface CommitData {
  id: string;
  repository_id: number;
  repository_name: string;
  author: string;
  email: string;
  message: string;
  timestamp: string;
  additions: number;
  deletions: number;
  files_changed: number;
  branch?: string;
}

export interface FileChange {
  path: string;
  additions: number;
  deletions: number;
  diff: string;
}

export interface CommitDetail extends CommitData {
  file_changes: FileChange[];
}

export interface HourlyStats {
  hour: number;
  additions: number;
  deletions: number;
  commits: number;
}

export interface DailyStats {
  date: string;
  additions: number;
  deletions: number;
  commits: number;
}

export interface WeeklyStats {
  weekday: number; // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
  additions: number;
  deletions: number;
  commits: number;
}

export interface HourlyCommitDistribution {
  hour: number;
  day_of_week: number; // 0 = Sunday, 1 = Monday, ..., 6 = Saturday
  commits: number;
}

export interface AuthorActivityTrend {
  author: string;
  period: string; // Format: "YYYY-MM" for monthly
  commits: number;
  additions: number;
  deletions: number;
}

export interface CommitFrequencyDistribution {
  date: string;
  commit_count: number;
}

export interface Statistics {
  hourly: HourlyStats[];
  daily: DailyStats[];
  weekly: WeeklyStats[];
  total_commits: number;
  total_additions: number;
  total_deletions: number;
  authors: { [key: string]: { additions: number; deletions: number; commits: number } };
  repositories: { [key: string]: { additions: number; deletions: number; commits: number } };
  // New fields for additional charts
  hourly_commit_distribution: HourlyCommitDistribution[];
  author_activity_trends: AuthorActivityTrend[];
  commit_frequency_distribution: CommitFrequencyDistribution[];
}

export interface TimeFilter {
  start_date?: string;
  end_date?: string;
  author?: string;
  repository_id?: number;
}