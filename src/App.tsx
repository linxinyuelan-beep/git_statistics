import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { useLocation } from 'react-router-dom';
import RepositoryManager from './components/RepositoryManager';
import StatisticsCharts from './components/StatisticsCharts';
import Timeline from './components/Timeline';
import { Repository, CommitData, Statistics, TimeFilter } from './types';

function App() {
  const location = useLocation();
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [statistics, setStatistics] = useState<Statistics | null>(null);
  const [timeline, setTimeline] = useState<CommitData[]>([]);
  const [allAuthors, setAllAuthors] = useState<string[]>([]); // 添加这行来存储所有作者
  const [loading, setLoading] = useState(false);
  const [loadingProgress, setLoadingProgress] = useState<{current: number, total: number, message: string} | null>(null);
  const [activeTab, setActiveTab] = useState<'charts' | 'timeline'>(() => {
    const savedTab = localStorage.getItem('activeTab');
    return savedTab === 'timeline' ? 'timeline' : 'charts';
  });
  const [filter, setFilter] = useState<TimeFilter>(() => {
    // 从 localStorage 恢复筛选条件
    const savedFilter = localStorage.getItem('git-stats-filter');
    if (savedFilter) {
      try {
        return JSON.parse(savedFilter);
      } catch (e) {
        console.error('解析筛选条件失败:', e);
        return {};
      }
    }
    
    // 默认设置为最近30天
    const thirtyDaysAgo = new Date();
    thirtyDaysAgo.setDate(thirtyDaysAgo.getDate() - 30);
    return {
      start_date: thirtyDaysAgo.toISOString().split('T')[0],
      exclude_authors: []
    };
  });
  const [sidebarCollapsed, setSidebarCollapsed] = useState<boolean>(() => {
    const saved = localStorage.getItem('sidebarCollapsed');
    return saved === 'true';
  });

  // 当从提交详情页面返回时，自动切换到时间线标签页并恢复筛选条件
  useEffect(() => {
    const isReturningFromCommitDetail = sessionStorage.getItem('returning-from-commit-detail') === 'true';
    if (isReturningFromCommitDetail) {
      console.log('从提交详情页面返回，切换到时间线标签页');
      setActiveTab('timeline');
      localStorage.setItem('activeTab', 'timeline');
      
      // 恢复筛选条件
      const savedFilter = localStorage.getItem('git-stats-filter');
      if (savedFilter) {
        try {
          const parsedFilter = JSON.parse(savedFilter);
          setFilter(parsedFilter);
          console.log('恢复筛选条件:', parsedFilter);
        } catch (e) {
          console.error('解析筛选条件失败:', e);
        }
      }
      
      // 清除标记，避免重复触发
      sessionStorage.removeItem('returning-from-commit-detail');
      
      // 确保数据重新加载
      if (repositories.length > 0) {
        loadData();
      }
    }
  }, [location.pathname]); // 改为监听路径变化

  useEffect(() => {
    loadRepositories();
  }, []);

  // 初始化所有作者列表
  useEffect(() => {
    const loadAllAuthors = async () => {
      try {
        const allCommits = await invoke<CommitData[]>('get_commit_timeline', {
          startDate: undefined,
          endDate: undefined,
          author: undefined,
          repositoryId: undefined
        });
        setAllAuthors(Array.from(new Set(allCommits.map(c => c.author))).sort());
      } catch (error) {
        console.error('Failed to load all authors:', error);
      }
    };
    
    loadAllAuthors();
  }, []);

  // 保存筛选条件到 localStorage
  useEffect(() => {
    localStorage.setItem('git-stats-filter', JSON.stringify(filter));
  }, [filter]);

  useEffect(() => {
    if (repositories.length > 0) {
      loadData();
    }
  }, [filter, repositories.length]);

  const loadData = async () => {
    try {
      // Convert date strings to ISO format for backend
      const startDate = filter.start_date ? new Date(filter.start_date + 'T00:00:00.000Z').toISOString() : undefined;
      const endDate = filter.end_date ? new Date(filter.end_date + 'T23:59:59.999Z').toISOString() : undefined;
      
      const [stats, timelineData] = await Promise.all([
        invoke<Statistics>('get_statistics', {
          startDate,
          endDate,
          author: filter.author,
          excludeAuthors: filter.exclude_authors,
          repositoryId: filter.repository_id
        }),
        invoke<CommitData[]>('get_commit_timeline', {
          startDate,
          endDate,
          author: filter.author,
          excludeAuthors: filter.exclude_authors,
          repositoryId: filter.repository_id
        })
      ]);
      
      setStatistics(stats);
      setTimeline(timelineData);
      
      // 更新所有作者列表（不考虑当前筛选条件）
      const allCommits = await invoke<CommitData[]>('get_commit_timeline', {
        startDate: undefined,
        endDate: undefined,
        author: undefined,
        excludeAuthors: undefined,
        repositoryId: filter.repository_id // 只考虑仓库筛选
      });
      
      setAllAuthors(Array.from(new Set(allCommits.map(c => c.author))).sort());
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  };

  const loadRepositories = async () => {
    try {
      const repos = await invoke<Repository[]>('get_repositories');
      setRepositories(repos);
    } catch (error) {
      console.error('Failed to load repositories:', error);
    }
  };

  const handleAddRepository = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      
      if (selected) {
        console.log('Selected directory:', selected);
        await invoke('add_repository', { path: selected });
        await loadRepositories();
      }
    } catch (error) {
      console.error('Failed to add repository:', error);
      // Show error to user
      alert(`添加仓库失败: ${error}`);
    }
  };

  const handleRemoveRepository = async (id: number) => {
    try {
      await invoke('remove_repository', { id });
      await loadRepositories();
    } catch (error) {
      console.error('Failed to remove repository:', error);
    }
  };

  

  const handleForceRefreshData = async () => {
    setLoading(true);
    setLoadingProgress({ current: 0, total: repositories.length, message: '开始全量刷新...' });
    
    try {
      // Force scan all repositories (full scan)
      for (let i = 0; i < repositories.length; i++) {
        const repo = repositories[i];
        // Update progress
        setLoadingProgress({ 
          current: i, 
          total: repositories.length, 
          message: `正在刷新仓库: ${repo.name} (${i+1}/${repositories.length})` 
        });
        
        // For long-running repository scans, show fake progress
        let fakeProgressCurrent = 0;
        const fakeProgressTotal = 100;
        const fakeProgressInterval = setInterval(() => {
          if (fakeProgressCurrent < fakeProgressTotal - 1) {
            fakeProgressCurrent += 1;
            setLoadingProgress({ 
              current: i + (fakeProgressCurrent / fakeProgressTotal), 
              total: repositories.length, 
              message: `正在刷新仓库: ${repo.name} - ${fakeProgressCurrent}%` 
            });
          }
        }, 50);
        
        try {
          await invoke('force_scan_repository', { repositoryId: repo.id });
        } finally {
          clearInterval(fakeProgressInterval);
        }
      }
      
      setLoadingProgress({ 
        current: repositories.length, 
        total: repositories.length, 
        message: '刷新完成，正在加载数据...' 
      });
      
      // Reload data with current filters
      await loadData();
    } catch (error) {
      console.error('Failed to force refresh data:', error);
    } finally {
      setLoading(false);
      setLoadingProgress(null);
    }
  };

  const handleRefreshLast24Hours = async () => {
    setLoading(true);
    setLoadingProgress({ current: 0, total: repositories.length, message: '开始刷新过去一天数据...' });
    
    try {
      // Scan last 24 hours for all repositories
      for (let i = 0; i < repositories.length; i++) {
        const repo = repositories[i];
        // Update progress
        setLoadingProgress({ 
          current: i, 
          total: repositories.length, 
          message: `正在刷新仓库: ${repo.name} (${i+1}/${repositories.length})` 
        });
        
        // For long-running repository scans, show fake progress
        let fakeProgressCurrent = 0;
        const fakeProgressTotal = 100;
        const fakeProgressInterval = setInterval(() => {
          if (fakeProgressCurrent < fakeProgressTotal - 1) {
            fakeProgressCurrent += 1;
            setLoadingProgress({ 
              current: i + (fakeProgressCurrent / fakeProgressTotal), 
              total: repositories.length, 
              message: `正在刷新仓库: ${repo.name} - ${fakeProgressCurrent}%` 
            });
          }
        }, 50);
        
        try {
          await invoke('scan_last_24_hours', { repositoryId: repo.id });
        } finally {
          clearInterval(fakeProgressInterval);
        }
      }
      
      setLoadingProgress({ 
        current: repositories.length, 
        total: repositories.length, 
        message: '刷新完成，正在加载数据...' 
      });
      
      // Reload data with current filters
      await loadData();
    } catch (error) {
      console.error('Failed to refresh last 24 hours data:', error);
    } finally {
      setLoading(false);
      setLoadingProgress(null);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-top">
          <h1>Commit 统计</h1>
          <div className="header-actions">
            <button onClick={handleRefreshLast24Hours} disabled={loading}>
              {loading ? '分析中...' : '刷新一天'}
            </button>
            <button onClick={handleForceRefreshData} disabled={loading}>
              {loading ? '分析中...' : '全量刷新'}
            </button>
          </div>
        </div>
        {loadingProgress && (
          <div className="progress-container">
            <div className="progress-info">{loadingProgress.message}</div>
            <div className="progress-bar">
              <div 
                className="progress-fill" 
                style={{ width: `${(loadingProgress.current / loadingProgress.total) * 100}%` }}
              ></div>
            </div>
            <div className="progress-text">{loadingProgress.current}/{loadingProgress.total}</div>
          </div>
        )}
        <div className="filter-section">
          <div className="filter-group">
            <label>开始日期:</label>
            <input
              type="date"
              value={filter.start_date || ''}
              onChange={(e) => setFilter(prev => ({ ...prev, start_date: e.target.value || undefined }))}
            />
          </div>
          <div className="filter-group">
            <label>结束日期:</label>
            <input
              type="date"
              value={filter.end_date || ''}
              onChange={(e) => setFilter(prev => ({ ...prev, end_date: e.target.value || undefined }))}
            />
          </div>
          <div className="filter-group">
            <label>作者:</label>
            <select
              value={filter.author || ''}
              onChange={(e) => setFilter(prev => ({ ...prev, author: e.target.value || undefined }))}
            >
              <option value="">所有作者</option>
              {allAuthors.map(author => (
                <option key={author} value={author}>{author}</option>
              ))}
            </select>
          </div>
          <div className="filter-group">
            <label>排除作者:</label>
            <select
              value={filter.exclude_authors?.[0] || ''}
              onChange={(e) => setFilter(prev => ({
                ...prev,
                exclude_authors: e.target.value ? [e.target.value] : []
              }))}
            >
              <option value="">不排除</option>
              {allAuthors.map(author => (
                <option key={author} value={author}>{author}</option>
              ))}
            </select>
          </div>
          <div className="filter-group">
            <label>仓库:</label>
            <select
              value={filter.repository_id || ''}
              onChange={(e) => setFilter(prev => ({ ...prev, repository_id: e.target.value ? Number(e.target.value) : undefined }))}
            >
              <option value="">全部仓库</option>
              {repositories.map(repo => (
                <option key={repo.id} value={repo.id}>{repo.name}</option>
              ))}
            </select>
          </div>
          <div className="quick-filters">
            <button onClick={() => setFilter(prev => ({ 
              ...prev,
              start_date: new Date().toISOString().split('T')[0],
              end_date: new Date().toISOString().split('T')[0]
            }))}>
              今日
            </button>
            <button onClick={() => setFilter(prev => ({ 
              ...prev,
              start_date: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString().split('T')[0],
              end_date: undefined
            }))}>
              昨天
            </button>
            <button onClick={() => setFilter(prev => ({ 
              ...prev,
              start_date: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
              end_date: undefined
            }))}>
              过去7天
            </button>
            <button onClick={() => setFilter(prev => ({ 
              ...prev,
              start_date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
              end_date: undefined
            }))}>
              过去30天
            </button>
            <button onClick={() => setFilter({})}>
              清除筛选
            </button>
          </div>
        </div>
      </header>

      <div className="app-content">
        <aside className={`sidebar ${sidebarCollapsed ? 'collapsed' : ''}`}>
          {!sidebarCollapsed && (
            <RepositoryManager
              repositories={repositories}
              onAdd={handleAddRepository}
              onRemove={handleRemoveRepository}
            />
          )}
          <button 
            className="sidebar-toggle" 
            onClick={() => {
              const newState = !sidebarCollapsed;
              setSidebarCollapsed(newState);
              localStorage.setItem('sidebarCollapsed', newState.toString());
            }}
            title={sidebarCollapsed ? '展开侧边栏' : '折叠侧边栏'}
          >
            {sidebarCollapsed ? '▶' : '◀'}
          </button>
        </aside>

        <main className="main-content">
          <div className="tabs">
            <button 
              className={activeTab === 'charts' ? 'active' : ''}
              onClick={() => {
                setActiveTab('charts');
                localStorage.setItem('activeTab', 'charts');
              }}
            >
              统计图表
            </button>
            <button 
              className={activeTab === 'timeline' ? 'active' : ''}
              onClick={() => {
                setActiveTab('timeline');
                localStorage.setItem('activeTab', 'timeline');
              }}
            >
              提交时间线
            </button>
          </div>

          <div className="tab-content">
            {activeTab === 'charts' && (
              <StatisticsCharts statistics={statistics} filter={filter} />
            )}
            {activeTab === 'timeline' && (
              <Timeline 
                commits={timeline} 
                filter={{
                  searchTerm: filter.searchTerm
                }}
                onFilterChange={(newFilter) => {
                  setFilter(prev => ({
                    ...prev,
                    searchTerm: newFilter.searchTerm
                  }));
                }}
              />
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;