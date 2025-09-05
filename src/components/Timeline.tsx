import React, { useState, useEffect, useRef } from 'react';
import { CommitData } from '../types';
import { useNavigate } from 'react-router-dom';

interface TimelineProps {
  commits: CommitData[];
  filter: {
    searchTerm?: string;
  };
  onFilterChange: (newFilter: { searchTerm?: string }) => void;
}

const Timeline: React.FC<TimelineProps> = ({ commits, filter, onFilterChange }) => {
  const navigate = useNavigate();
  const timelineRef = useRef<HTMLDivElement>(null);
  
  // 使用传入的筛选条件
  const { searchTerm = '' } = filter;
  
  // 统一折叠状态管理
  const [collapsed, setCollapsed] = useState(() => {
    const saved = localStorage.getItem('timeline-collapsed');
    return saved === 'true';
  });
  
  // 保存折叠状态到 localStorage
  const toggleCollapse = () => {
    const newState = !collapsed;
    setCollapsed(newState);
    localStorage.setItem('timeline-collapsed', newState.toString());
  };
  
  // 更新筛选条件的函数
  const handleSearchChange = (term: string) => {
    onFilterChange({ searchTerm: term });
  };
  
  // 使用 localStorage 保存和恢复滚动位置
  useEffect(() => {
    const handleScroll = () => {
      if (timelineRef.current) {
        // 实时保存滚动位置到 localStorage
        const scrollTop = timelineRef.current.scrollTop;
        localStorage.setItem('timeline-scroll-position', scrollTop.toString());
        console.log('保存滚动位置:', scrollTop);
      }
    };
    
    const container = timelineRef.current;
    if (container) {
      container.addEventListener('scroll', handleScroll);
      return () => container.removeEventListener('scroll', handleScroll);
    }
  }, []);
  
  // 恢复滚动位置
  useEffect(() => {
    // 检查是否是从提交详情页面返回的
    const shouldRestore = sessionStorage.getItem('will-return-from-commit-detail') === 'true';
    console.log('检查是否需要恢复滚动位置:', shouldRestore);
    
    if (shouldRestore && timelineRef.current) {
      // 从 localStorage 恢复滚动位置
      const savedScrollPosition = localStorage.getItem('timeline-scroll-position');
      console.log('获取保存的滚动位置:', savedScrollPosition);
      
      if (savedScrollPosition) {
        // 稍微延迟一下，确保 DOM 已经渲染完成
        setTimeout(() => {
          if (timelineRef.current) {
            const scrollTop = parseInt(savedScrollPosition, 10);
            timelineRef.current.scrollTop = scrollTop;
            console.log('恢复滚动位置到:', scrollTop, '实际位置:', timelineRef.current.scrollTop);
          }
        }, 100);
      }
      
      // 清除标记，避免重复触发
      sessionStorage.removeItem('will-return-from-commit-detail');
    }
  }, [commits]); // 当commits数据加载完成时触发

  // 在这里处理条件渲染
  if (!commits) {
    return (
      <div className="empty-state">
        <h3>加载中...</h3>
        <p>正在加载提交数据</p>
      </div>
    );
  }
  
  if (commits.length === 0) {
    return (
      <div className="empty-state">
        <h3>暂无提交数据</h3>
        <p>请先添加仓库并刷新数据</p>
      </div>
    );
  }

  const filteredCommits = commits.filter(commit => {
    const matchesSearch = !searchTerm || 
      commit.message.toLowerCase().includes(searchTerm.toLowerCase()) ||
      commit.author.toLowerCase().includes(searchTerm.toLowerCase());
    
    return matchesSearch;
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
    // 在跳转前保存当前滚动位置
    if (timelineRef.current) {
      const scrollTop = timelineRef.current.scrollTop;
      localStorage.setItem('timeline-scroll-position', scrollTop.toString());
      // 设置标记表示将要进入commit详情页面
      sessionStorage.setItem('will-return-from-commit-detail', 'true');
      console.log('点击commit时保存滚动位置:', scrollTop);
    }
    // Navigate to commit detail page
    navigate(`/commit/${commit.repository_id}/${commit.id}`);
  };

  return (
    <div className="timeline-container">
      <div className="timeline-section-header" onClick={toggleCollapse}>
        <h3>筛选和统计 {collapsed ? '▶' : '▼'}</h3>
      </div>
      {!collapsed && (
        <>
          <div className="timeline-filters">
            <div className="filter-group">
              <input
                type="text"
                placeholder="搜索提交消息或作者..."
                value={searchTerm}
                onChange={(e) => handleSearchChange(e.target.value)}
                className="search-input"
              />
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
        </>
      )}

      <div className="timeline-list" ref={timelineRef}>
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
    </div>
  );
};

export default Timeline;