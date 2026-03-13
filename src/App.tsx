import { useServer } from './hooks/useServer';
import { DirectorySelector } from './components/DirectorySelector';
import { ServerControl } from './components/ServerControl';
import { QRCodeDisplay } from './components/QRCodeDisplay';
import { AlertCircle } from 'lucide-react';
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

  const handleSelectDirectory = async (path: string) => {
    try {
      await setSharedDir(path);
    } catch (err) {
      console.error('Failed to set directory:', err);
    }
  };

  const handleClearDirectory = async () => {
    // 如果服务器在运行，先停止
    if (status.isRunning) {
      await stopServer();
    }
    // 清除目录（通过设置空路径或重新加载页面）
    window.location.reload();
  };

  const handleStartServer = async () => {
    try {
      await startServer();
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
      <header className="app-header">
        <h1>📁 CrossFlow</h1>
        <p>局域网文件传输助手</p>
      </header>

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
              <li>点击"启动服务"开启文件服务器</li>
              <li>使用手机扫描二维码访问 Web 界面</li>
              <li>在手机上可以浏览、下载或上传文件</li>
              <li>完成后点击"停止服务"关闭服务器</li>
            </ol>
          </div>
        )}
      </main>
    </div>
  );
}

export default App;