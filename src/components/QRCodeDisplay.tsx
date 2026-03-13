import React, { useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { ServerInfo } from '../types';
import { Copy, Check, Wifi } from 'lucide-react';

interface QRCodeDisplayProps {
  serverInfo: ServerInfo;
}

export const QRCodeDisplay: React.FC<QRCodeDisplayProps> = ({ serverInfo }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(serverInfo.url);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  return (
    <div className="qr-display">
      <div className="qr-header">
        <Wifi size={20} />
        <span>扫码访问</span>
      </div>
      
      <div className="qr-container">
        <QRCodeSVG
          value={serverInfo.url}
          size={200}
          level="M"
          includeMargin={true}
          bgColor="#ffffff"
          fgColor="#000000"
        />
      </div>

      <div className="server-info">
        <div className="info-row">
          <span className="info-label">地址:</span>
          <span className="info-value">{serverInfo.address}</span>
        </div>
        <div className="info-row">
          <span className="info-label">端口:</span>
          <span className="info-value">{serverInfo.port}</span>
        </div>
      </div>

      <div className="url-display">
        <code className="url-text">{serverInfo.url}</code>
        <button
          className="btn btn-icon btn-small"
          onClick={handleCopy}
          title="复制链接"
        >
          {copied ? <Check size={16} className="text-success" /> : <Copy size={16} />}
        </button>
      </div>

      <p className="qr-hint">
        使用手机扫描二维码，或复制链接到浏览器访问
      </p>
    </div>
  );
};
