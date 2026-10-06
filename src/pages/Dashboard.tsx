import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppStatus, LogEvent } from '../types';
import ProgressBar from '../components/ProgressBar';
import LogViewer from '../components/LogViewer';

interface QuickAction {
  label: string;
  icon: string;
  page: 'secure-delete' | 'full-drive-erase' | 'artifact-scanner';
}

const quickActions: QuickAction[] = [
  { label: 'New Secure Delete', icon: '\u{1F5D1}\uFE0F', page: 'secure-delete' },
  { label: 'Full Drive Erase', icon: '\u26A1', page: 'full-drive-erase' },
  { label: 'Scan Artifacts', icon: '\u{1F50D}', page: 'artifact-scanner' },
];

interface DashboardProps {
  onNavigate: (page: 'secure-delete' | 'full-drive-erase' | 'artifact-scanner') => void;
}

export default function Dashboard({ onNavigate }: DashboardProps) {
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadStatus();
  }, []);

  async function loadStatus() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<AppStatus>('get_app_status');
      setStatus(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    invoke<LogEvent[]>('get_recent_logs', { limit: 50 })
      .then(setLogs)
      .catch(() => {});
  }, []);

  if (loading) {
    return (
      <div>
        <h1 className="page-title">Dashboard</h1>
        <p className="page-subtitle">System overview and quick actions</p>
        <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
          <div className="spinner" />
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <h1 className="page-title">Dashboard</h1>
        <div className="error-message">Failed to load system status: {error}</div>
        <button className="btn btn-primary" onClick={loadStatus}>Retry</button>
      </div>
    );
  }

  const statCardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    transition: 'background-color 0.3s ease, border-color 0.3s ease',
  };

  const healthIndicator = (healthy: boolean) => ({
    width: 10,
    height: 10,
    borderRadius: '50%',
    background: healthy ? 'var(--success)' : 'var(--danger)',
    display: 'inline-block',
    marginRight: 6,
  });

  const actionBtnStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: 10,
    padding: '14px 18px',
    background: 'var(--bg-tertiary)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    color: 'var(--text-primary)',
    fontSize: '0.875rem',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    width: '100%',
  };

  return (
    <div className="fade-in">
      <h1 className="page-title">Dashboard</h1>
      <p className="page-subtitle">System overview and quick actions</p>

      <div className="card-grid">
        <div style={statCardStyle}>
          <div className="stat-label">Total Devices</div>
          <div className="stat-value">{status?.totalOperations ?? 0}</div>
        </div>
        <div style={statCardStyle}>
          <div className="stat-label">Files Deleted</div>
          <div className="stat-value">{status?.totalFilesDeleted ?? 0}</div>
        </div>
        <div style={statCardStyle}>
          <div className="stat-label">Bytes Processed</div>
          <div className="stat-value">
            {formatBytes(status?.totalBytesProcessed ?? 0)}
          </div>
        </div>
        <div style={statCardStyle}>
          <div className="stat-label">Security Status</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
            <span style={healthIndicator((status?.errorsLast24h ?? 0) === 0)} />
            <span style={{ fontSize: '0.9rem', color: 'var(--text-secondary)' }}>
              {(status?.errorsLast24h ?? 0) === 0 ? 'Healthy' : `${status?.errorsLast24h} error(s)`}
            </span>
          </div>
        </div>
      </div>

      <div className="section">
        <h3 className="section-title">Quick Actions</h3>
        <div className="flex-wrap">
          {quickActions.map((action) => (
            <button
              key={action.page}
              style={actionBtnStyle}
              onClick={() => onNavigate(action.page)}
              onMouseEnter={(e) => {
                e.currentTarget.style.background = 'var(--border)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.background = 'var(--bg-tertiary)';
              }}
            >
              <span style={{ fontSize: '1.2rem' }}>{action.icon}</span>
              <span>{action.label}</span>
            </button>
          ))}
        </div>
      </div>

      <div className="section">
        <h3 className="section-title">System Health</h3>
        <div style={statCardStyle}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>Uptime</span>
              <span style={{ fontSize: '0.85rem', fontWeight: 500 }}>{formatUptime(status?.uptime ?? 0)}</span>
            </div>
            <ProgressBar
              value={100}
              label="System Health"
              statusText="All systems operational"
              color="var(--success)"
            />
          </div>
        </div>
      </div>

      <div className="section">
        <h3 className="section-title">Recent Activity</h3>
        <LogViewer entries={logs} maxEntries={50} />
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const parts = [];
  if (d > 0) parts.push(`${d}d`);
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  return parts.join(' ') || '<1m';
}
