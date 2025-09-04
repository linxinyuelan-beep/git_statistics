import React, { useState } from 'react';
import { CommitData } from '../types';

interface TimelineProps {
  commits: CommitData[];
}

const Timeline: React.FC<TimelineProps> = ({ commits }) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedAuthor, setSelectedAuthor] = useState('');
  const [selectedCommit, setSelectedCommit] = useState<CommitData | null>(null);

  if (!commits || commits.length === 0) {
    return (
      <div className="empty-state">
        <h3>暂无提交数据</h3>
        <p>请先添加仓库并刷新数据</p>
      </div>
    );
  }

  const authors = Array.from(new Set(commits.map(c => c.author))).sort();
  
  const filteredCommits = commits.filter(commit => {
    const matchesSearch = !searchTerm || 
      commit.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      commit.author.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesAuthor = !selectedAuthor || commit.author === selectedAuthor;
    
    return matchesSearch && matchesAuthor;
  });

  const formatDate = (timestamp: string) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const handleCommitClick = (commit: CommitData) => {
    setSelectedCommit(commit);
  };

  const closeCommitDetail = () => {
    setSelectedCommit(null);
  };

  return (
    <div className="timeline-container">
      <div className="timeline-filters">
        <div className="filter-group">
          <input
            type="text"
            placeholder="搜索提交消息或作者..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="search-input"
          />
        </div>
        
        <div className="filter-group">
          <select
            value={selectedAuthor}
            onChange={(e) => setSelectedAuthor(e.target.value)}
            className="author-select"
          >
            <option value="">所有作者</option>
            {authors.map(author => (
              <option key={author} value={author}>{author}</option>
            ))}
          </select>
        </div>
      </div>

      <div className="timeline-stats">
        <div className="stat-item">
          <span className="stat-label">显示提交:</span>
          <span className="stat-value">{filteredCommits.length}</span>
        </div>
        <div className="stat-item">
          <span className="stat-label">总新增:</span>
          <span className="stat-value text-green">
            +{filteredCommits.reduce((sum, c) => sum + c.additions, 0)}
          </span>
        </div>
        <div className="stat-item">
          <span className="stat-label">总删除:</span>
          <span className="stat-value text-red">
            -{filteredCommits.reduce((sum, c) => sum + c.deletions, 0)}
          </span>
        </div>
      </div>

      <div className="timeline-list">
        {filteredCommits.map((commit) => (
          <div 
            key={`${commit.repository_id}-${commit.id}`} 
            className="timeline-item"
            onClick={() => handleCommitClick(commit)}
          >
            <div className="commit-meta">
              <div className="commit-author">
                <strong>{commit.author}</strong>
                <span className="repository-badge">{commit.repository_name}</span>
                {commit.branch && (
                  <span className="branch-badge">{commit.branch}</span>
                )}
              </div>
              <div className="commit-time">{formatDate(commit.timestamp)}</div>
            </div>
            
            <div className="commit-message">{commit.message}</div>
            
            <div className="commit-stats">
              <span className="stat-changes">
                <span className="text-green">+{commit.additions}</span>
                {' '}
                <span className="text-red">-{commit.deletions}</span>
              </span>
              <span className="stat-files">
                {commit.files_changed} 个文件修改
              </span>
            </div>
          </div>
        ))}
      </div>

      {/* Commit Detail Modal */}
      {selectedCommit && (
        <div className="modal-overlay" onClick={closeCommitDetail}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>提交详情</h2>
              <button className="modal-close" onClick={closeCommitDetail}>×</button>
            </div>
            <div className="modal-body">
              <div className="commit-detail-info">
                <div className="detail-row">
                  <span className="detail-label">作者:</span>
                  <span className="detail-value">{selectedCommit.author}</span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">邮箱:</span>
                  <span className="detail-value">{selectedCommit.email}</span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">时间:</span>
                  <span className="detail-value">{formatDate(selectedCommit.timestamp)}</span>
                </div>
                <div className="detail-row">
                  <span className="detail-label">仓库:</span>
                  <span className="detail-value">{selectedCommit.repository_name}</span>
                </div>
                {selectedCommit.branch && (
                  <div className="detail-row">
                    <span className="detail-label">分支:</span>
                    <span className="detail-value">{selectedCommit.branch}</span>
                  </div>
                )}
                <div className="detail-row">
                  <span className="detail-label">提交ID:</span>
                  <span className="detail-value commit-id">{selectedCommit.id}</span>
                </div>
              </div>
              
              <div className="commit-detail-message">
                <h3>提交信息</h3>
                <pre>{selectedCommit.message}</pre>
              </div>
              
              <div className="commit-detail-stats">
                <h3>变更统计</h3>
                <div className="stats-grid">
                  <div className="stat-card">
                    <div className="stat-number text-green">+{selectedCommit.additions}</div>
                    <div className="stat-label">新增行数</div>
                  </div>
                  <div className="stat-card">
                    <div className="stat-number text-red">-{selectedCommit.deletions}</div>
                    <div className="stat-label">删除行数</div>
                  </div>
                  <div className="stat-card">
                    <div className="stat-number">{selectedCommit.files_changed}</div>
                    <div className="stat-label">文件变更</div>
                  </div>
                </div>
              </div>
              
              {/* TODO: Add file changes and diff viewer here */}
              <div className="commit-detail-files">
                <h3>文件变更</h3>
                <p>文件变更详情和代码差异查看功能将在后续版本中实现。</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default Timeline;