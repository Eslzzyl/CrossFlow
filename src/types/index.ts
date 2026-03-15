export interface FileInfo {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number;
  modified?: string;
}

export interface DirectoryListing {
  current_path: string;
  parent_path?: string;
  files: FileInfo[];
}

export interface ServerInfo {
  address: string;
  port: number;
  url: string;
}

export interface ServerStatus {
  isRunning: boolean;
  serverInfo?: ServerInfo;
  sharedDir?: string;
  password?: string;
}

export type DeviceType = 'mobile' | 'tablet' | 'desktop' | 'unknown';

export interface Device {
  id: string;
  ip: string;
  user_agent: string;
  device_name: string;
  device_type: DeviceType;
  first_seen: number;
  last_seen: number;
}
