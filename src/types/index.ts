export interface DeviceInfo {
  devicePath: string;
  model: string;
  size: number;
  interfaceType: string;
  isSystemDisk: boolean;
  healthStatus: string;
}

export interface DeviceDetails {
  devicePath: string;
  model: string;
  serial: string;
  firmwareRevision: string;
  size: number;
  sectorSize: number;
  interfaceType: string;
  isSystemDisk: boolean;
  healthStatus: string;
  temperature: number | null;
  nvmeVersion: string | null;
  trimSupported: boolean;
  ataVersion: string | null;
  cryptoCapable: boolean;
}

export interface DriveEraseReport {
  devicePath: string;
  model: string;
  size: number;
  eraseMode: string;
  startTime: string;
  endTime: string;
  duration: number;
  passesCompleted: number;
  totalPasses: number;
  bytesWritten: number;
  verified: boolean;
  verificationPassed: boolean;
  status: string;
  error: string | null;
}

export interface DeletionReport {
  id: string;
  targetPath: string;
  securityLevel: number;
  startTime: string;
  endTime: string;
  duration: number;
  filesProcessed: number;
  directoriesProcessed: number;
  totalSize: number;
  bytesOverwritten: number;
  verified: boolean;
  verificationPassed: boolean;
  trimExecuted: boolean;
  status: string;
  error: string | null;
}

export interface VerificationResult {
  timestamp: string;
  targetPath: string;
  expectedHash: string;
  actualHash: string;
  match: boolean;
  confidence: number;
  method: string;
}

export interface TrimResult {
  devicePath: string;
  trimmed: boolean;
  bytesTrimmed: number;
  duration: number;
  error: string | null;
}

export interface ScanFinding {
  path: string;
  size: number;
  riskLevel: string;
  description: string;
  artifactType: string;
  lastModified: string;
}

export interface ScanResult {
  id: string;
  scanPath: string;
  startTime: string;
  endTime: string;
  duration: number;
  totalFiles: number;
  totalFindings: number;
  findingsByType: Record<string, number>;
  findingsByRisk: Record<string, number>;
  findings: ScanFinding[];
}

export interface ReportData {
  id: string;
  reportType: string;
  generatedAt: string;
  sections: string[];
  filePath: string;
  fileSize: number;
  data: string;
}

export interface Settings {
  theme: string;
  defaultSecurityLevel: number;
  overwritePasses: number;
  verifyAfterDelete: boolean;
  trimAfterDelete: boolean;
  autoGenerateReport: boolean;
  notificationLevel: string;
  adminMode: boolean;
}

export interface AppStatus {
  version: string;
  uptime: number;
  totalOperations: number;
  totalFilesDeleted: number;
  totalBytesProcessed: number;
  errorsLast24h: number;
}

export interface ProgressEvent {
  phase: string;
  progress: number;
  message: string;
  currentItem: string;
  processedItems: number;
  totalItems: number;
  speed: string;
  eta: string;
}

export interface LogEvent {
  timestamp: string;
  level: string;
  message: string;
}

export type Page = 'dashboard' | 'devices' | 'secure-delete' | 'full-drive-erase' | 'artifact-scanner' | 'verification' | 'reports' | 'settings';
