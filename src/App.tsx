import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import RepositoryManager from './components/RepositoryManager';
import StatisticsCharts from './components/StatisticsCharts';
import Timeline from './components/Timeline';
import { Repository, CommitData, Statistics, TimeFilter } from './types';

function App() {
  const [repositories, setRepositories] = useState<Repository[]>([]);
  const [statistics, setStatistics] = useState<Statistics | null>(null);
  const [timeline, setTimeline] = useState<CommitData[]>([]);
  const [loading, setLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'charts' | 'timeline'>('charts');
  const [filter, setFilter] = useState<TimeFilter>({});

  useEffect(() => {
    loadRepositories();
  }, []);

  useEffect(() => {
    if (repositories.length > 0) {
      loadData();
    }
  }, [filter]);

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
          repositoryId: filter.repository_id
        }),
        invoke<CommitData[]>('get_commit_timeline', {
          startDate,
          endDate,
          author: filter.author,
          repositoryId: filter.repository_id
        })
      ]);
      
      setStatistics(stats);
      setTimeline(timelineData);
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

  const handleRefreshData = async () => {
    setLoading(true);
    try {
      // Scan all repositories
      for (const repo of repositories) {
        await invoke('scan_repository', { repositoryId: repo.id });
      }
      
      // Reload data with current filters
      await loadData();
    } catch (error) {
      console.error('Failed to refresh data:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-top">
          <h1>Git 工作量统计</h1>
          <div className="header-actions">
            <button onClick={handleRefreshData} disabled={loading}>
              {loading ? '分析中...' : '刷新数据'}
            </button>
          </div>
        </div>
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
            <input
              type="text"
              placeholder="输入作者名"
              value={filter.author || ''}
              onChange={(e) => setFilter(prev => ({ ...prev, author: e.target.value || undefined }))}
            />
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
            <button onClick={() => setFilter(prev => ({ ...prev, start_date: new Date(Date.now() - 24 * 60 * 60 * 1000).toISOString().split('T')[0], end_date: undefined }))}>
              昨天
            </button>
            <button onClick={() => setFilter(prev => ({ ...prev, start_date: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], end_date: undefined }))}>
              过去7天
            </button>
            <button onClick={() => setFilter(prev => ({ ...prev, start_date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], end_date: undefined }))}>
              过去30天
            </button>
            <button onClick={() => setFilter({})}>
              清除筛选
            </button>
          </div>
        </div>
      </header>

      <div className="app-content">
        <aside className="sidebar">
          <RepositoryManager
            repositories={repositories}
            onAdd={handleAddRepository}
            onRemove={handleRemoveRepository}
          />
        </aside>

        <main className="main-content">
          <div className="tabs">
            <button 
              className={activeTab === 'charts' ? 'active' : ''}
              onClick={() => setActiveTab('charts')}
            >
              统计图表
            </button>
            <button 
              className={activeTab === 'timeline' ? 'active' : ''}
              onClick={() => setActiveTab('timeline')}
            >
              提交时间线
            </button>
          </div>

          <div className="tab-content">
            {activeTab === 'charts' && (
              <StatisticsCharts statistics={statistics} />
            )}
            {activeTab === 'timeline' && (
              <Timeline commits={timeline} />
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;