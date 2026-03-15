import { invoke } from '@tauri-apps/api/core';
import { Device, ServerInfo, ServerStatus } from '../types';

export const api = {
  // 设置共享目录
  setSharedDir: async (path: string): Promise<void> => {
    await invoke('set_shared_dir', { path });
  },

  // 获取当前共享目录
  getSharedDir: async (): Promise<string | null> => {
    return await invoke('get_shared_dir');
  },

  // 清除共享目录
  clearSharedDir: async (): Promise<void> => {
    await invoke('clear_shared_dir');
  },

  // 启动服务器
  startServer: async (port?: number, password?: string): Promise<ServerInfo> => {
    return await invoke('start_server', { port, password });
  },

  // 停止服务器
  stopServer: async (): Promise<void> => {
    await invoke('stop_server');
  },

  // 获取服务器状态
  getServerStatus: async (): Promise<ServerStatus> => {
    const serverInfo = await invoke<ServerInfo | null>('get_server_status');
    const sharedDir = await invoke<string | null>('get_shared_dir');

    return {
      isRunning: !!serverInfo,
      serverInfo: serverInfo || undefined,
      sharedDir: sharedDir || undefined,
    };
  },

  // 生成 QR 码
  generateQRCode: async (data: string, size?: number): Promise<string> => {
    return await invoke('generate_qr_code', { data, size });
  },

  // 获取服务器地址列表
  getServerAddresses: async (port: number): Promise<ServerInfo[]> => {
    return await invoke('get_server_addresses', { port });
  },

  // 获取已连接的设备列表
  getConnectedDevices: async (): Promise<Device[]> => {
    return await invoke('get_connected_devices');
  },
};
