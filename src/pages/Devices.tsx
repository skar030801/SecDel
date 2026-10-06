import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DeviceInfo, DeviceDetails } from '../types';

export default function Devices() {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [expandedDevice, setExpandedDevice] = useState<string | null>(null);
  const [details, setDetails] = useState<Record<string, DeviceDetails>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actionLoading, setActionLoading] = useState<string | null>(null);
  const [actionMessage, setActionMessage] = useState<string | null>(null);

  useEffect(() => {
    loadDevices();
  }, []);

  async function loadDevices() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<DeviceInfo[]>('get_devices');
      setDevices(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  async function toggleExpand(devicePath: string) {
    if (expandedDevice === devicePath) {
      setExpandedDevice(null);
      return;
    }
    setExpandedDevice(devicePath);
    if (!details[devicePath]) {
      try {
        const deviceDetails = await invoke<DeviceDetails>('get_device_info', { path: devicePath });
        setDetails((prev) => ({ ...prev, [devicePath]: deviceDetails }));
      } catch {
        setDetails((prev) => ({ ...prev, [devicePath]: null as unknown as DeviceDetails }));
      }
    }
  }

  async function performAction(devicePath: string, action: string) {
    setActionLoading(devicePath);
    setActionMessage(null);
    try {
      const result = await invoke<string>(action, { devicePath });
      setActionMessage(`${action} completed: ${result}`);
    } catch (err) {
      setActionMessage(`Error: ${String(err)}`);
    } finally {
      setActionLoading(null);
    }
  }

  function getHealthBadge(health: string) {
    const variant = health.toLowerCase() === 'good' || health.toLowerCase() === 'healthy'
      ? 'badge-success'
      : health.toLowerCase() === 'caution' || health.toLowerCase() === 'warning'
        ? 'badge-warning'
        : 'badge-danger';
    return <span className={`badge ${variant}`}>{health}</span>;
  }

  if (loading) {
    return (
      <div>
        <h1 className="page-title">Devices</h1>
        <p className="page-subtitle">Detected storage devices</p>
        <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
          <div className="spinner" />
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div>
        <h1 className="page-title">Devices</h1>
        <div className="error-message">Failed to load devices: {error}</div>
        <button className="btn btn-primary" onClick={loadDevices}>Retry</button>
      </div>
    );
  }

  const headerRowStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 24,
  };

  const detailPanelStyle: React.CSSProperties = {
    background: 'var(--bg-tertiary)',
    borderRadius: 'var(--radius)',
    padding: 16,
    marginTop: 8,
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: 12,
    fontSize: '0.85rem',
  };

  const detailItemStyle: React.CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: 2,
  };

  const detailLabelStyle: React.CSSProperties = {
    fontSize: '0.72rem',
    color: 'var(--text-muted)',
    textTransform: 'uppercase',
    letterSpacing: '0.3px',
  };

  const detailValueStyle: React.CSSProperties = {
    color: 'var(--text-primary)',
    fontWeight: 500,
  };

  return (
    <div className="fade-in">
      <div style={headerRowStyle}>
        <div>
          <h1 className="page-title">Devices</h1>
          <p className="page-subtitle">Detected storage devices ({devices.length})</p>
        </div>
        <button className="btn btn-primary" onClick={loadDevices}>
          Refresh
        </button>
      </div>

      {actionMessage && (
        <div className={actionMessage.startsWith('Error') ? 'error-message' : 'success-message'}>
          {actionMessage}
        </div>
      )}

      {devices.length === 0 ? (
        <div className="empty-state">
          <div className="empty-state-icon">{'\u{1F4BE}'}</div>
          <div className="empty-state-text">No storage devices detected</div>
        </div>
      ) : (
        <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
          <table className="table">
            <thead>
              <tr>
                <th>Device</th>
                <th>Model</th>
                <th>Size</th>
                <th>Interface</th>
                <th>Health</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {devices.map((device) => (
                <>
                  <tr
                    key={device.devicePath}
                    onClick={() => toggleExpand(device.devicePath)}
                    style={{ cursor: 'pointer' }}
                  >
                    <td style={{ fontWeight: 500 }}>{device.devicePath}</td>
                    <td>{device.model}</td>
                    <td>{formatBytes(device.size)}</td>
                    <td>{device.interfaceType}</td>
                    <td>{getHealthBadge(device.healthStatus)}</td>
                    <td>
                      <div className="flex-row" onClick={(e) => e.stopPropagation()}>
                        <button
                          className="btn btn-sm btn-secondary"
                          onClick={() => performAction(device.devicePath, 'erase_device')}
                          disabled={actionLoading === device.devicePath}
                        >
                          {actionLoading === device.devicePath ? (
                            <span className="spinner" style={{ width: 14, height: 14 }} />
                          ) : (
                            'Erase'
                          )}
                        </button>
                        <button
                          className="btn btn-sm btn-secondary"
                          onClick={() => performAction(device.devicePath, 'sanitize_device')}
                          disabled={actionLoading === device.devicePath}
                        >
                          Sanitize
                        </button>
                      </div>
                    </td>
                  </tr>
                  {expandedDevice === device.devicePath && (
                    <tr key={`${device.devicePath}-details`}>
                      <td colSpan={6} style={{ padding: '0 16px 16px' }}>
                        {details[device.devicePath] ? (
                          <div style={detailPanelStyle}>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>Serial</span>
                              <span style={detailValueStyle}>{details[device.devicePath].serial}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>Firmware</span>
                              <span style={detailValueStyle}>{details[device.devicePath].firmwareRevision}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>Sector Size</span>
                              <span style={detailValueStyle}>{details[device.devicePath].sectorSize} bytes</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>NVMe Version</span>
                              <span style={detailValueStyle}>{details[device.devicePath].nvmeVersion || 'N/A'}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>TRIM Support</span>
                              <span style={detailValueStyle}>{details[device.devicePath].trimSupported ? 'Yes' : 'No'}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>ATA Version</span>
                              <span style={detailValueStyle}>{details[device.devicePath].ataVersion || 'N/A'}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>Crypto Capable</span>
                              <span style={detailValueStyle}>{details[device.devicePath].cryptoCapable ? 'Yes' : 'No'}</span>
                            </div>
                            <div style={detailItemStyle}>
                              <span style={detailLabelStyle}>Temperature</span>
                              <span style={detailValueStyle}>
                                {details[device.devicePath].temperature != null
                                  ? `${details[device.devicePath].temperature}\u00B0C`
                                  : 'N/A'}
                              </span>
                            </div>
                          </div>
                        ) : (
                          <div style={{ display: 'flex', justifyContent: 'center', padding: 16 }}>
                            <span className="spinner" />
                          </div>
                        )}
                      </td>
                    </tr>
                  )}
                </>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${units[i]}`;
}
