import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/tauri';
import { CommitDetail, FileChange } from '../types';
import { PrismLight as SyntaxHighlighter } from 'react-syntax-highlighter';
import { prism } from 'react-syntax-highlighter/dist/esm/styles/prism';
import diff from 'react-syntax-highlighter/dist/esm/languages/prism/diff';

// 注册语言
SyntaxHighlighter.registerLanguage('diff', diff);

const CommitDetailPage: React.FC = () => {
  const { repositoryId, commitId } = useParams<{ repositoryId: string; commitId: string }>();
  const navigate = useNavigate();
  const [commitDetail, setCommitDetail] = useState<CommitDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchCommitDetail = async () => {
      if (!repositoryId || !commitId) {
        setError('无效的提交信息');
        setLoading(false);
        return;
      }

      try {
        setLoading(true);
        const detail: CommitDetail = await invoke('get_commit_detail', {
          repositoryId: parseInt(repositoryId),
          commitId
        });
        setCommitDetail(detail);
      } catch (err) {
        console.error('Failed to fetch commit detail:', err);
        setError('获取提交详情失败');
      } finally {
        setLoading(false);
      }
    };

    fetchCommitDetail();
  }, [repositoryId, commitId]);

  const formatDate = (timestamp: string) => {
    return new Date(timestamp).toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  // Render file changes with diff
  const renderFileChanges = (fileChanges: FileChange[]) => {
    if (!fileChanges || fileChanges.length === 0) {
      return <p>没有文件变更信息。</p>;
    }

    return (
      <div className="file-changes-list">
        {fileChanges.map((fileChange, index) => (
          <div key={index} className="file-change-item">
            <div className="file-change-header">
              <span className="file-path">{fileChange.path}</span>
              <div className="file-stats">
                <span className="stat-additions">+{fileChange.additions}</span>
                <span className="stat-deletions">-{fileChange.deletions}</span>
              </div>
            </div>
            <div className="file-diff">
              <SyntaxHighlighter 
                language="diff" 
                style={prism}
                customStyle={{
                  margin: 0,
                  padding: '10px',
                  fontSize: '0.85rem',
                  lineHeight: '1.4'
                }}
                wrapLines={true}
                showLineNumbers={true}
              >
                {fileChange.diff}
              </SyntaxHighlighter>
            </div>
          </div>
        ))}
      </div>
    );
  };

  if (loading) {
    return (
      <div className="commit-detail-page">
        <div className="page-header">
          <button className="back-button" onClick={() => {
            // 设置返回标记
            sessionStorage.setItem('returning-from-commit-detail', 'true');
            navigate('/');
          }}>← 返回</button>
          <h1>提交详情</h1>
        </div>
        <div className="page-content">
          <div className="loading">正在加载提交详情...</div>
        </div>
      </div>
    );
  }

  if (error || !commitDetail) {
    return (
      <div className="commit-detail-page">
        <div className="page-header">
          <button className="back-button" onClick={() => {
            // 设置返回标记
            sessionStorage.setItem('returning-from-commit-detail', 'true');
            navigate('/');
          }}>← 返回</button>
          <h1>提交详情</h1>
        </div>
        <div className="page-content">
          <div className="error">{error || '无法加载提交详情'}</div>
        </div>
      </div>
    );
  }

  return (
    <div className="commit-detail-page">
      <div className="page-header">
        <button className="back-button" onClick={() => {
          // 设置返回标记
          sessionStorage.setItem('returning-from-commit-detail', 'true');
          navigate(-1);
        }}>← 返回</button>
        <h1>提交详情</h1>
      </div>
      
      <div className="page-content">
        <div className="commit-detail-info">
          <div className="detail-row">
            <span className="detail-label">作者:</span>
            <span className="detail-value">{commitDetail.author}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">邮箱:</span>
            <span className="detail-value">{commitDetail.email}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">时间:</span>
            <span className="detail-value">{formatDate(commitDetail.timestamp)}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">仓库:</span>
            <span className="detail-value">{commitDetail.repository_name}</span>
          </div>
          {commitDetail.branch && (
            <div className="detail-row">
              <span className="detail-label">分支:</span>
              <span className="detail-value">{commitDetail.branch}</span>
            </div>
          )}
          <div className="detail-row">
            <span className="detail-label">提交ID:</span>
            <span className="detail-value commit-id">{commitDetail.id}</span>
          </div>
        </div>
        
        <div className="commit-detail-message">
          <h3>提交信息</h3>
          <pre>{commitDetail.message}</pre>
        </div>
        
        <div className="commit-detail-stats">
          <h3>变更统计</h3>
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-number text-green">+{commitDetail.additions}</div>
              <div className="stat-label">新增行数</div>
            </div>
            <div className="stat-card">
              <div className="stat-number text-red">-{commitDetail.deletions}</div>
              <div className="stat-label">删除行数</div>
            </div>
            <div className="stat-card">
              <div className="stat-number">{commitDetail.files_changed}</div>
              <div className="stat-label">文件变更</div>
            </div>
          </div>
        </div>
        
        <div className="commit-detail-files">
          <h3>文件变更</h3>
          {renderFileChanges(commitDetail.file_changes)}
        </div>
      </div>
    </div>
  );
};

export default CommitDetailPage;