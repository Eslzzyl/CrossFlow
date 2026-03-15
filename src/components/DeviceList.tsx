import { useState, useEffect, useCallback } from 'react';
import { Smartphone, Tablet, Monitor, HelpCircle, RefreshCw, Wifi } from 'lucide-react';
import { Device, DeviceType } from '../types';
import { api } from '../services/api';

interface DeviceListProps {
  isRunning: boolean;
}

const deviceTypeIcons: Record<DeviceType, React.ReactNode> = {
  mobile: <Smartphone size={20} />,
  tablet: <Tablet size={20} />,
  desktop: <Monitor size={20} />,
  unknown: <HelpCircle size={20} />,
};

const deviceTypeLabels: Record<DeviceType, string> = {
  mobile: '手机',
  tablet: '平板',
  desktop: '电脑',
  unknown: '未知设备',
};

function formatTime(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  const diff = now.getTime() - date.getTime();

  // 小于1分钟
  if (diff < 60 * 1000) {
    return '刚刚';
  }
  // 小于1小时
  if (diff < 60 * 60 * 1000) {
    return `${Math.floor(diff / (60 * 1000))}分钟前`;
  }
  // 小于24小时
  if (diff < 24 * 60 * 60 * 1000) {
    return `${Math.floor(diff / (60 * 60 * 1000))}小时前`;
  }

  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function DeviceList({ isRunning }: DeviceListProps) {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchDevices = useCallback(async () => {
    if (!isRunning) {
      setDevices([]);
      return;
    }

    try {
      const deviceList = await api.getConnectedDevices();
      setDevices(deviceList);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : '获取设备列表失败');
    }
  }, [isRunning]);

  // 自动刷新设备列表
  useEffect(() => {
    fetchDevices();

    if (!isRunning) return;

    // 每3秒刷新一次
    const interval = setInterval(fetchDevices, 3000);
    return () => clearInterval(interval);
  }, [fetchDevices, isRunning]);

  const handleRefresh = async () => {
    setLoading(true);
    await fetchDevices();
    setLoading(false);
  };

  if (!isRunning) {
    return null;
  }

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '16px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Wifi size={18} style={{ color: '#10b981' }} />
          <span style={{ fontWeight: 600, color: 'var(--text-primary)' }}>
            已连接设备 ({devices.length})
          </span>
        </div>
        <button
          onClick={handleRefresh}
          disabled={loading}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '4px',
            padding: '6px 12px',
            border: '1px solid #e5e7eb',
            borderRadius: '6px',
            background: 'white',
            cursor: loading ? 'not-allowed' : 'pointer',
            fontSize: '13px',
            color: 'var(--text-secondary)',
            opacity: loading ? 0.6 : 1,
          }}
        >
          <RefreshCw size={14} style={{ animation: loading ? 'spin 1s linear infinite' : 'none' }} />
          刷新
        </button>
      </div>

      {error && (
        <div style={{
          padding: '12px',
          background: '#fef2f2',
          borderRadius: '8px',
          color: '#dc2626',
          fontSize: '14px',
          marginBottom: '12px',
        }}>
          {error}
        </div>
      )}

      {devices.length === 0 ? (
        <div style={{
          padding: '32px',
          textAlign: 'center',
          color: 'var(--text-secondary)',
          background: '#f8fafc',
          borderRadius: '10px',
        }}>
          <Wifi size={32} style={{ marginBottom: '12px', opacity: 0.3 }} />
          <p style={{ margin: 0, fontSize: '14px' }}>暂无设备连接</p>
          <p style={{ margin: '4px 0 0 0', fontSize: '12px', opacity: 0.7 }}>
            使用手机扫描二维码即可连接
          </p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
          {devices.map((device) => (
            <div
              key={device.id}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                padding: '14px 16px',
                background: '#f8fafc',
                borderRadius: '10px',
                transition: 'all 0.2s ease',
              }}
            >
              <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                width: '40px',
                height: '40px',
                borderRadius: '10px',
                background: 'white',
                color: 'var(--primary-color)',
                boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
              }}>
                {deviceTypeIcons[device.device_type]}
              </div>

              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  marginBottom: '2px',
                }}>
                  <span style={{
                    fontWeight: 600,
                    color: 'var(--text-primary)',
                    fontSize: '14px',
                  }}>
                    {device.device_name}
                  </span>
                  <span style={{
                    padding: '2px 6px',
                    background: '#e0e7ff',
                    color: '#4f46e5',
                    borderRadius: '4px',
                    fontSize: '11px',
                    fontWeight: 500,
                  }}>
                    {deviceTypeLabels[device.device_type]}
                  </span>
                </div>
                <div style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  fontSize: '12px',
                  color: 'var(--text-secondary)',
                }}>
                  <span>IP: {device.ip}</span>
                  <span>•</span>
                  <span>活跃: {formatTime(device.last_seen)}</span>
                </div>
              </div>

              <div style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                background: '#10b981',
                boxShadow: '0 0 0 2px #d1fae5',
              }} />
            </div>
          ))}
        </div>
      )}

      <style>{`
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
