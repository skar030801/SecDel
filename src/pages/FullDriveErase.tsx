import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DeviceInfo, DriveEraseReport, ProgressEvent } from '../types';
import ProgressBar from '../components/ProgressBar';

interface EraseMode {
  value: string;
  label: string;
  description: string;
}

const eraseModes: EraseMode[] = [
  { value: 'quick', label: 'Quick', description: 'Single pass overwrite. Fast, suitable for SSDs.' },
  { value: 'standard', label: 'Standard', description: 'Two passes with random data. Balanced security.' },
  { value: 'secure', label: 'Secure', description: 'Three passes per DoD 5220.22-M standard.' },
  { value: 'ultimate', label: 'Ultimate', description: 'Seven passes with cryptographic random data.' },
];

export default function FullDriveErase() {
  const [devices, setDevices] = useState<DeviceInfo[]>([]);
  const [selectedDevice, setSelectedDevice] = useState('');
  const [eraseMode, setEraseMode] = useState('standard');
  const [acknowledged, setAcknowledged] = useState(false);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [result, setResult] = useState<DriveEraseReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

  useEffect(() => {
    invoke<DeviceInfo[]>('get_devices')
      .then(setDevices)
      .catch(() => {});
  }, []);

  useEffect(() => {
    const setup = async () => {
      unlistenRef.current = await listen<ProgressEvent>('progress', (event) => {
        setProgress(event.payload);
      });
    };
    setup();
    return () => {
      if (unlistenRef.current) unlistenRef.current();
    };
  }, []);

  const startErase = async () => {
    if (!selectedDevice || !acknowledged) return;
    setRunning(true);
    setError(null);
    setResult(null);
    setProgress(null);
    try {
      const modeNum = { quick: 0, standard: 1, secure: 2, ultimate: 3 }[eraseMode] ?? 1;
      const report = await invoke<DriveEraseReport>('erase_drive', {
        device: selectedDevice,
        mode: modeNum,
      });
      setResult(report);
    } catch (err) {
      setError(String(err));
    } finally {
      setRunning(false);
    }
  };

  const selectedDeviceInfo = devices.find((d) => d.devicePath === selectedDevice);

  const cardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    marginBottom: 16,
  };

  return (
    <div className="fade-in">
      <h1 className="page-title">Full Drive Erase</h1>
      <p className="page-subtitle">Securely erase entire storage devices</p>

      {error && <div className="error-message">{error}</div>}
      {result && (
        <div className={result.status === 'completed' ? 'success-message' : 'error-message'}>
          {result.status === 'completed'
            ? `Drive erase completed: ${result.passesCompleted}/${result.totalPasses} passes, ${formatBytes(result.bytesWritten)} written`
            : `Erase failed: ${result.error || result.status}`}
        </div>
      )}

      <div style={{ maxWidth: 700 }}>
        <div style={cardStyle}>
          <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Select Device</h3>
          <div className="form-group">
            <label className="form-label">Storage Device</label>
            <select
              className="select"
              value={selectedDevice}
              onChange={(e) => setSelectedDevice(e.target.value)}
              disabled={running}
            >
              <option value="">-- Select a device --</option>
              {devices.map((device) => (
                <option key={device.devicePath} value={device.devicePath}>
                  {device.devicePath} - {device.model} ({formatBytes(device.size)})
                </option>
              ))}
            </select>
          </div>
          {selectedDeviceInfo && (
            <div style={{ display: 'flex', gap: 16, flexWrap: 'wrap', fontSize: '0.82rem', color: 'var(--text-secondary)' }}>
              <span>Model: {selectedDeviceInfo.model}</span>
              <span>Interface: {selectedDeviceInfo.interfaceType}</span>
              <span>Health: {selectedDeviceInfo.healthStatus}</span>
            </div>
          )}
        </div>

        <div style={cardStyle}>
          <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Erase Mode</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
            {eraseModes.map((mode) => (
              <label
                key={mode.value}
                style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: 10,
                  padding: 10,
                  borderRadius: 'var(--radius-sm)',
                  background: eraseMode === mode.value ? 'var(--accent-light)' : 'transparent',
                  cursor: 'pointer',
                }}
              >
                <input
                  type="radio"
                  name="eraseMode"
                  value={mode.value}
                  checked={eraseMode === mode.value}
                  onChange={() => setEraseMode(mode.value)}
                  style={{ marginTop: 3, accentColor: 'var(--accent)' }}
                  disabled={running}
                />
                <div>
                  <div style={{ fontWeight: 500, fontSize: '0.85rem' }}>{mode.label}</div>
                  <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>{mode.description}</div>
                </div>
              </label>
            ))}
          </div>
        </div>

        <div style={cardStyle}>
          <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Warning</h3>
          <div className="warning-box">
            <div className="warning-box-title">Irreversible Action</div>
            <div className="warning-box-text">
              This will permanently erase all data on the selected device. This action cannot be undone.
              Ensure you have backed up any important data before proceeding.
              {selectedDeviceInfo?.isSystemDisk && (
                <div style={{ marginTop: 8, fontWeight: 600, color: 'var(--danger)' }}>
                  WARNING: This appears to be a system disk. Erasing it may render your system unbootable.
                </div>
              )}
            </div>
          </div>
          <label className="checkbox-label">
            <input
              type="checkbox"
              checked={acknowledged}
              onChange={(e) => setAcknowledged(e.target.checked)}
              disabled={running}
            />
            <span style={{ fontSize: '0.85rem' }}>
              I understand that all data on this device will be permanently destroyed
            </span>
          </label>
        </div>

        <button
          className="btn btn-danger"
          style={{ width: '100%', justifyContent: 'center', padding: 14, fontSize: '1rem' }}
          onClick={startErase}
          disabled={running || !selectedDevice || !acknowledged}
        >
          {running ? (
            <><span className="spinner" style={{ width: 18, height: 18 }} /> Erasing...</>
          ) : (
            <>{'\u26A1'} Start Full Drive Erase</>
          )}
        </button>

        {(running || progress) && (
          <div style={{ ...cardStyle, marginTop: 16 }}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Progress</h3>
            {progress && (
              <ProgressBar
                value={progress.progress}
                label={progress.phase}
                statusText={progress.message}
              />
            )}
            {progress && (
              <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.78rem', color: 'var(--text-muted)' }}>
                <span>{progress.processedItems} / {progress.totalItems}</span>
                <span>{progress.speed} | ETA: {progress.eta}</span>
              </div>
            )}
          </div>
        )}

        {result && (
          <div style={cardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Result</h3>
            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, fontSize: '0.85rem' }}>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Status</div>
                <div style={{ fontWeight: 500 }}>{result.status}</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Mode</div>
                <div style={{ fontWeight: 500 }}>{result.eraseMode}</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Passes</div>
                <div style={{ fontWeight: 500 }}>{result.passesCompleted} / {result.totalPasses}</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Bytes Written</div>
                <div style={{ fontWeight: 500 }}>{formatBytes(result.bytesWritten)}</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Duration</div>
                <div style={{ fontWeight: 500 }}>{result.duration}s</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Verification</div>
                <div style={{ fontWeight: 500 }}>{result.verified ? (result.verificationPassed ? 'Passed' : 'Failed') : 'N/A'}</div>
              </div>
            </div>
          </div>
        )}
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
