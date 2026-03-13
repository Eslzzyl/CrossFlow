import { useState, useEffect, useCallback } from 'react';
import { api } from '../services/api';
import { ServerInfo, ServerStatus } from '../types';

export function useServer() {
  const [status, setStatus] = useState<ServerStatus>({
    isRunning: false,
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 刷新状态
  const refreshStatus = useCallback(async () => {
    try {
      const newStatus = await api.getServerStatus();
      setStatus(newStatus);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  }, []);

  // 初始加载状态
  useEffect(() => {
    refreshStatus();
  }, [refreshStatus]);

  // 设置共享目录
  const setSharedDir = useCallback(async (path: string) => {
    setLoading(true);
    setError(null);
    try {
      await api.setSharedDir(path);
      await refreshStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to set shared directory');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [refreshStatus]);

  // 启动服务器
  const startServer = useCallback(async (port?: number, password?: string): Promise<ServerInfo> => {
    setLoading(true);
    setError(null);
    try {
      const serverInfo = await api.startServer(port, password);
      await refreshStatus();
      return serverInfo;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to start server';
      setError(message);
      throw err;
    } finally {
      setLoading(false);
    }
  }, [refreshStatus]);

  // 停止服务器
  const stopServer = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      await api.stopServer();
      await refreshStatus();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to stop server');
      throw err;
    } finally {
      setLoading(false);
    }
  }, [refreshStatus]);

  return {
    status,
    loading,
    error,
    setSharedDir,
    startServer,
    stopServer,
    refreshStatus,
  };
}
