import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type { ProgressEvent, LogEvent, DeletionReport } from '../types';
import ProgressBar from '../components/ProgressBar';
import LogViewer from '../components/LogViewer';

interface SelectedFile {
  path: string;
  size: number;
}

const securityLevels = [
  { value: 1, label: 'Level 1 - Quick', description: 'Single pass overwrite with zeros. Fast but less secure.' },
  { value: 2, label: 'Level 2 - Standard', description: 'Two passes: random data then zeros. Good for most uses.' },
  { value: 3, label: 'Level 3 - Secure', description: 'Three passes: random, zeros, random. Meets DoD 5220.22-M.' },
  { value: 4, label: 'Level 4 - Ultimate', description: 'Seven passes with multiple patterns. Maximum security.' },
];

export default function SecureDelete() {
  const [selectedFiles, setSelectedFiles] = useState<SelectedFile[]>([]);
  const [securityLevel, setSecurityLevel] = useState(2);
  const [verifyEnabled, setVerifyEnabled] = useState(true);
  const [trimEnabled, setTrimEnabled] = useState(true);
  const [createReport, setCreateReport] = useState(true);
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<ProgressEvent | null>(null);
  const [logs, setLogs] = useState<LogEvent[]>([]);
  const [result, setResult] = useState<DeletionReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const unlistenRef = useRef<UnlistenFn | null>(null);

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

  const selectFiles = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'All Files', extensions: ['*'] }],
      });
      if (selected) {
        const filesWithSizes: SelectedFile[] = [];
        for (const path of selected) {
          try {
            const size = await invoke<number>('get_file_size', { path });
            filesWithSizes.push({ path, size });
          } catch {
            filesWithSizes.push({ path, size: 0 });
          }
        }
        setSelectedFiles((prev) => [...prev, ...filesWithSizes]);
      }
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const removeFile = (path: string) => {
    setSelectedFiles((prev) => prev.filter((f) => f.path !== path));
  };

  const clearFiles = () => {
    setSelectedFiles([]);
  };

  const startDelete = async () => {
    if (selectedFiles.length === 0) return;
    setRunning(true);
    setError(null);
    setResult(null);
    setProgress(null);
    setLogs([]);
    try {
      const report = await invoke<DeletionReport>('delete_files', {
        paths: selectedFiles.map((f) => f.path),
        level: securityLevel,
      });
      setResult(report);
      setLogs((prev) => [...prev, {
        timestamp: new Date().toISOString(),
        level: 'success',
        message: `Deletion completed: ${report.filesProcessed} files processed`,
      }]);
    } catch (err) {
      setError(String(err));
      setLogs((prev) => [...prev, {
        timestamp: new Date().toISOString(),
        level: 'error',
        message: String(err),
      }]);
    } finally {
      setRunning(false);
    }
  };

  const containerStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    gap: 24,
  };

  const optionsCardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
  };

  const fileItemStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '8px 12px',
    background: 'var(--bg-tertiary)',
    borderRadius: 'var(--radius-sm)',
    fontSize: '0.8rem',
    marginBottom: 4,
  };

  const summaryStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    marginTop: 24,
  };

  const summaryRowStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px 0',
    borderBottom: '1px solid var(--border)',
    fontSize: '0.85rem',
  };

  return (
    <div className="fade-in">
      <h1 className="page-title">Secure Delete</h1>
      <p className="page-subtitle">Delete files with configurable overwrite and verification options</p>

      {error && <div className="error-message">{error}</div>}
      {result && (
        <div className="success-message">
          Successfully deleted {result.filesProcessed} files ({formatBytes(result.totalSize)})
          {result.verified && result.verificationPassed && ' - Verified'}
        </div>
      )}

      <div style={containerStyle}>
        <div>
          <div style={optionsCardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Files & Folders</h3>
            <div className="flex-row mb-16">
              <button className="btn btn-primary" onClick={selectFiles} disabled={running}>
                Select Files
              </button>
              {selectedFiles.length > 0 && (
                <button className="btn btn-secondary" onClick={clearFiles} disabled={running}>
                  Clear All
                </button>
              )}
            </div>
            {selectedFiles.length > 0 && (
              <div style={{ maxHeight: 200, overflowY: 'auto' }}>
                {selectedFiles.map((file) => (
                  <div key={file.path} style={fileItemStyle}>
                    <span style={{ flex: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                      {file.path}
                    </span>
                    <span style={{ color: 'var(--text-muted)', margin: '0 8px', whiteSpace: 'nowrap' }}>
                      {formatBytes(file.size)}
                    </span>
                    <button
                      style={{ background: 'none', border: 'none', color: 'var(--danger)', cursor: 'pointer', fontSize: '1rem', padding: 0 }}
                      onClick={() => removeFile(file.path)}
                      disabled={running}
                    >
                      &times;
                    </button>
                  </div>
                ))}
              </div>
            )}
            {selectedFiles.length === 0 && (
              <div style={{ color: 'var(--text-muted)', fontSize: '0.85rem', textAlign: 'center', padding: 24 }}>
                No files selected. Click "Select Files" to choose files for secure deletion.
              </div>
            )}
          </div>

          <div style={{ ...optionsCardStyle, marginTop: 16 }}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Security Level</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              {securityLevels.map((level) => (
                <label
                  key={level.value}
                  style={{
                    display: 'flex',
                    alignItems: 'flex-start',
                    gap: 10,
                    padding: 10,
                    borderRadius: 'var(--radius-sm)',
                    background: securityLevel === level.value ? 'var(--accent-light)' : 'transparent',
                    cursor: 'pointer',
                    transition: 'background 0.2s ease',
                  }}
                >
                  <input
                    type="radio"
                    name="securityLevel"
                    value={level.value}
                    checked={securityLevel === level.value}
                    onChange={() => setSecurityLevel(level.value)}
                    style={{ marginTop: 3, accentColor: 'var(--accent)' }}
                    disabled={running}
                  />
                  <div>
                    <div style={{ fontWeight: 500, fontSize: '0.85rem', color: 'var(--text-primary)' }}>{level.label}</div>
                    <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>{level.description}</div>
                  </div>
                </label>
              ))}
            </div>
          </div>
        </div>

        <div>
          <div style={optionsCardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Options</h3>
            <div className="checkbox-group">
              <label className="checkbox-label">
                <input type="checkbox" checked={verifyEnabled} onChange={(e) => setVerifyEnabled(e.target.checked)} disabled={running} />
                Verify after deletion
              </label>
              <label className="checkbox-label">
                <input type="checkbox" checked={trimEnabled} onChange={(e) => setTrimEnabled(e.target.checked)} disabled={running} />
                Execute TRIM after deletion
              </label>
              <label className="checkbox-label">
                <input type="checkbox" checked={createReport} onChange={(e) => setCreateReport(e.target.checked)} disabled={running} />
                Create report
              </label>
            </div>
          </div>

          <div style={{ marginTop: 16 }}>
            <button
              className="btn btn-danger"
              style={{ width: '100%', justifyContent: 'center', padding: 14 }}
              onClick={startDelete}
              disabled={running || selectedFiles.length === 0}
            >
              {running ? (
                <><span className="spinner" style={{ width: 18, height: 18 }} /> Processing...</>
              ) : (
                <>{'\u{1F5D1}\uFE0F'} Start Secure Deletion</>
              )}
            </button>
          </div>

          {(running || progress) && (
            <div style={{ ...optionsCardStyle, marginTop: 16 }}>
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
                  <span>{progress.processedItems} / {progress.totalItems} items</span>
                  <span>{progress.speed} | ETA: {progress.eta}</span>
                </div>
              )}
            </div>
          )}
        </div>
      </div>

      {(running || logs.length > 0) && (
        <div style={{ marginTop: 24 }}>
          <h3 className="section-title">Log</h3>
          <LogViewer entries={logs} maxEntries={200} />
        </div>
      )}

      {result && (
        <div style={summaryStyle}>
          <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Result Summary</h3>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Status</span>
            <span style={{ fontWeight: 600, color: 'var(--success)' }}>{result.status}</span>
          </div>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Files Processed</span>
            <span>{result.filesProcessed}</span>
          </div>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Total Size</span>
            <span>{formatBytes(result.totalSize)}</span>
          </div>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Duration</span>
            <span>{result.duration}s</span>
          </div>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Verification</span>
            <span>{result.verified ? (result.verificationPassed ? 'Passed' : 'Failed') : 'N/A'}</span>
          </div>
          <div style={summaryRowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>TRIM</span>
            <span>{result.trimExecuted ? 'Executed' : 'N/A'}</span>
          </div>
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
