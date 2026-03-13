import React from 'react';
import { Folder, FolderOpen, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';

interface DirectorySelectorProps {
  path: string | null;
  onSelect: (path: string) => void;
  onClear: () => void;
  disabled?: boolean;
}

export const DirectorySelector: React.FC<DirectorySelectorProps> = ({
  path,
  onSelect,
  onClear,
  disabled = false,
}) => {
  const handleSelect = async () => {
    try {
      // 使用 Tauri 的 dialog API 选择目录
      const selected = await invoke<string | null>('select_directory');
      if (selected) {
        onSelect(selected);
      }
    } catch (err) {
      console.error('Failed to select directory:', err);
    }
  };

  return (
    <div className="directory-selector">
      {path ? (
        <div className="selected-path">
          <div className="path-info">
            <FolderOpen size={20} />
            <span className="path-text" title={path}>
              {path}
            </span>
          </div>
          {!disabled && (
            <button className="btn-icon" onClick={onClear} title="清除">
              <X size={18} />
            </button>
          )}
        </div>
      ) : (
        <button
          className="btn btn-secondary btn-full"
          onClick={handleSelect}
          disabled={disabled}
        >
          <Folder size={18} />
          选择共享目录
        </button>
      )}
    </div>
  );
};
