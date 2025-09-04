import React from 'react';
import { Repository } from '../types';

interface RepositoryManagerProps {
  repositories: Repository[];
  onAdd: () => void;
  onRemove: (id: number) => void;
}

const RepositoryManager: React.FC<RepositoryManagerProps> = ({
  repositories,
  onAdd,
  onRemove,
}) => {
  return (
    <div className="repository-manager">
      <h3>仓库管理</h3>
      
      <button className="add-repo-btn" onClick={onAdd}>
        + 添加仓库
      </button>

      <div className="repository-list">
        {repositories.length === 0 ? (
          <div className="empty-state">
            <p>暂无仓库</p>
            <p>点击上方按钮添加 Git 仓库</p>
          </div>
        ) : (
          repositories.map((repo) => (
            <div key={repo.id} className="repository-item">
              <div>
                <div className="repository-name">{repo.name}</div>
                <div className="repository-path">{repo.path}</div>
                {repo.last_scanned && (
                  <div className="last-scanned">
                    最后扫描: {new Date(repo.last_scanned).toLocaleString('zh-CN')}
                  </div>
                )}
              </div>
              <button
                className="remove-btn"
                onClick={() => onRemove(repo.id)}
              >
                删除
              </button>
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default RepositoryManager;