import React from 'react';
import { Play, Square, Loader2 } from 'lucide-react';

interface ServerControlProps {
  isRunning: boolean;
  loading: boolean;
  hasSharedDir: boolean;
  onStart: () => void;
  onStop: () => void;
}

export const ServerControl: React.FC<ServerControlProps> = ({
  isRunning,
  loading,
  hasSharedDir,
  onStart,
  onStop,
}) => {
  return (
    <div className="server-control">
      <div className="status-indicator">
        <div className={`status-dot ${isRunning ? 'active' : 'inactive'}`} />
        <span className="status-text">
          {isRunning ? '服务运行中' : '服务已停止'}
        </span>
      </div>

      {isRunning ? (
        <button
          className="btn btn-danger"
          onClick={onStop}
          disabled={loading}
        >
          {loading ? (
            <Loader2 size={18} className="spin" />
          ) : (
            <Square size={18} />
          )}
          停止服务
        </button>
      ) : (
        <button
          className="btn btn-primary"
          onClick={onStart}
          disabled={loading || !hasSharedDir}
        >
          {loading ? (
            <Loader2 size={18} className="spin" />
          ) : (
            <Play size={18} />
          )}
          启动服务
        </button>
      )}
    </div>
  );
};
