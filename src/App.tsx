import { useState } from 'react';
import { useServer } from './hooks/useServer';
import { DirectorySelector } from './components/DirectorySelector';
import { ServerControl } from './components/ServerControl';
import { QRCodeDisplay } from './components/QRCodeDisplay';
import { AlertCircle, Lock } from 'lucide-react';
import './App.css';

function App() {
  const {
    status,
    loading,
    error,
    setSharedDir,
    startServer,
    stopServer,
  } = useServer();

  const [password, setPassword] = useState('');
  const [enablePassword, setEnablePassword] = useState(false);

  const handleSelectDirectory = async (path: string) => {
    try {
      await setSharedDir(path);
    } catch (err) {
      console.error('Failed to set directory:', err);
    }
  };

  const handleClearDirectory = async () => {
    if (status.isRunning) {
      await stopServer();
    }
    window.location.reload();
  };

  const handleStartServer = async () => {
    try {
      await startServer(undefined, enablePassword ? password : undefined);
    } catch (err) {
      console.error('Failed to start server:', err);
    }
  };

  const handleStopServer = async () => {
    try {
      await stopServer();
    } catch (err) {
      console.error('Failed to stop server:', err);
    }
  };

  return (
    <div className="app">
      <main className="app-main">
        {error && (
          <div className="error-message">
            <AlertCircle size={18} />
            <span>{error}</span>
          </div>
        )}

        {/* 目录选择 */}
        <div className="card">
          <h2 className="card-title">选择共享目录</h2>
          <DirectorySelector
            path={status.sharedDir || null}
            onSelect={handleSelectDirectory}
            onClear={handleClearDirectory}
            disabled={status.isRunning}
          />
        </div>

        {/* 服务器控制 */}
        <div className="card">
          <h2 className="card-title">服务器控制</h2>

          {/* 密码保护选项 */}
          {!status.isRunning && (
            <div style={{ marginBottom: '20px', padding: '16px', background: '#f8fafc', borderRadius: '10px' }}>
              <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer', marginBottom: enablePassword ? '12px' : '0' }}>
                <input
                  type="checkbox"
                  checked={enablePassword}
                  onChange={(e) => setEnablePassword(e.target.checked)}
                  disabled={status.isRunning}
                />
                <Lock size={16} />
                <span>启用密码保护</span>
              </label>

              {enablePassword && (
                <input
                  type="password"
                  placeholder="设置访问密码"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  disabled={status.isRunning}
                  style={{
                    width: '100%',
                    padding: '10px 14px',
                    border: '2px solid #e5e7eb',
                    borderRadius: '8px',
                    fontSize: '14px',
                    marginTop: '8px'
                  }}
                />
              )}
            </div>
          )}

          {status.isRunning && status.password && (
            <div style={{ marginBottom: '16px', padding: '12px', background: '#fef3c7', borderRadius: '8px', fontSize: '14px' }}>
              <Lock size={14} style={{ marginRight: '6px', verticalAlign: 'middle' }} />
              密码保护已启用
            </div>
          )}

          <ServerControl
            isRunning={status.isRunning}
            loading={loading}
            hasSharedDir={!!status.sharedDir}
            onStart={handleStartServer}
            onStop={handleStopServer}
          />
        </div>

        {/* QR 码显示 */}
        {status.isRunning && status.serverInfo && (
          <div className="card">
            <QRCodeDisplay serverInfo={status.serverInfo} />
          </div>
        )}

        {/* 使用说明 */}
        {!status.isRunning && (
          <div className="card">
            <h2 className="card-title">使用说明</h2>
            <ol style={{ paddingLeft: '20px', lineHeight: '2', color: 'var(--text-secondary)' }}>
              <li>点击"选择共享目录"选择要共享的文件夹</li>
              <li>（可选）启用密码保护，设置访问密码</li>
              <li>点击"启动服务"开启文件服务器</li>
              <li>使用手机扫描二维码访问 Web 界面</li>
              <li>在手机上可以浏览、下载、上传或删除文件</li>
              <li>完成后点击"停止服务"关闭服务器</li>
            </ol>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;