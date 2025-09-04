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

export interface Statistics {
  hourly: HourlyStats[];
  daily: DailyStats[];
  total_commits: number;
  total_additions: number;
  total_deletions: number;
  authors: { [key: string]: { additions: number; deletions: number; commits: number } };
  repositories: { [key: string]: { additions: number; deletions: number; commits: number } };
}

export interface TimeFilter {
  start_date?: string;
  end_date?: string;
  author?: string;
  repository_id?: number;
}