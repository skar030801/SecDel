import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { ScanResult, ScanFinding } from '../types';
import ProgressBar from '../components/ProgressBar';

export default function ArtifactScanner() {
  const [scanPath, setScanPath] = useState('');
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState<ScanResult | null>(null);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [expandedType, setExpandedType] = useState<string | null>(null);

  const selectPath = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });
      if (selected) {
        setScanPath(selected);
      }
    } catch (err) {
      setError(String(err));
    }
  };

  const startScan = async () => {
    if (!scanPath) return;
    setScanning(true);
    setError(null);
    setResult(null);
    setProgress(0);
    try {
      const scanResult = await invoke<ScanResult>('scan_artifacts', { paths: [scanPath] });
      setResult(scanResult);
    } catch (err) {
      setError(String(err));
    } finally {
      setScanning(false);
      setProgress(100);
    }
  };

  const exportResults = async () => {
    if (!result) return;
    try {
      const filePath = await invoke<string>('export_scan_results', { scanId: result.id });
      setError(`Results exported to: ${filePath}`);
    } catch (err) {
      setError(String(err));
    }
  };

  const getRiskBadge = (risk: string) => {
    const variant = risk === 'high' ? 'badge-danger' : risk === 'medium' ? 'badge-warning' : 'badge-info';
    return <span className={`badge ${variant}`}>{risk}</span>;
  };

  const groupedFindings: Record<string, ScanFinding[]> = {};
  if (result) {
    for (const finding of result.findings) {
      const type = finding.artifactType || 'Unknown';
      if (!groupedFindings[type]) groupedFindings[type] = [];
      groupedFindings[type].push(finding);
    }
  }

  const cardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    marginBottom: 16,
  };

  const findingItemStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '8px 12px',
    borderBottom: '1px solid var(--border)',
    fontSize: '0.82rem',
    cursor: 'pointer',
  };

  return (
    <div className="fade-in">
      <h1 className="page-title">Artifact Scanner</h1>
      <p className="page-subtitle">Scan for residual data and forensic artifacts</p>

      {error && (
        <div className={error.startsWith('Results exported') ? 'success-message' : 'error-message'}>
          {error}
        </div>
      )}

      <div style={{ maxWidth: 800 }}>
        <div style={cardStyle}>
          <div className="form-group">
            <label className="form-label">Scan Path</label>
            <div className="flex-row">
              <input
                className="input"
                type="text"
                value={scanPath}
                onChange={(e) => setScanPath(e.target.value)}
                placeholder="Select a folder to scan..."
                disabled={scanning}
              />
              <button className="btn btn-secondary" onClick={selectPath} disabled={scanning}>
                Browse
              </button>
            </div>
          </div>
          <button
            className="btn btn-primary"
            onClick={startScan}
            disabled={scanning || !scanPath}
          >
            {scanning ? (
              <><span className="spinner" style={{ width: 16, height: 16 }} /> Scanning...</>
            ) : (
              <>{'\u{1F50D}'} Start Scan</>
            )}
          </button>

          {scanning && (
            <div style={{ marginTop: 16 }}>
              <ProgressBar value={progress} variant="indeterminate" statusText="Scanning for artifacts..." />
            </div>
          )}
        </div>

        {result && (
          <>
            <div style={cardStyle}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
                <h3 style={{ fontSize: '0.95rem', fontWeight: 600 }}>Scan Summary</h3>
                <button className="btn btn-sm btn-secondary" onClick={exportResults}>
                  Export Results
                </button>
              </div>
              <div style={{ display: 'flex', gap: 24, flexWrap: 'wrap' }}>
                <div>
                  <div className="stat-label">Total Findings</div>
                  <div className="stat-value" style={{ fontSize: '1.5rem' }}>{result.totalFindings}</div>
                </div>
                <div>
                  <div className="stat-label">Files Scanned</div>
                  <div className="stat-value" style={{ fontSize: '1.5rem', color: 'var(--text-primary)' }}>{result.totalFiles}</div>
                </div>
                <div>
                  <div className="stat-label">Duration</div>
                  <div className="stat-value" style={{ fontSize: '1.5rem', color: 'var(--text-primary)' }}>{result.duration}s</div>
                </div>
              </div>
              {result.findingsByRisk && Object.keys(result.findingsByRisk).length > 0 && (
                <div style={{ marginTop: 16, display: 'flex', gap: 8 }}>
                  {Object.entries(result.findingsByRisk).map(([risk, count]) => (
                    <div key={risk} style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.82rem' }}>
                      {getRiskBadge(risk)}
                      <span style={{ fontWeight: 600 }}>{count}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            <div style={cardStyle}>
              <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Findings</h3>
              {Object.entries(groupedFindings).length === 0 ? (
                <div className="empty-state">
                  <div className="empty-state-text">No artifacts found</div>
                </div>
              ) : (
                Object.entries(groupedFindings).map(([type, findings]) => (
                  <div key={type} style={{ marginBottom: 8 }}>
                    <div
                      style={{
                        ...findingItemStyle,
                        background: 'var(--bg-tertiary)',
                        fontWeight: 600,
                        cursor: 'pointer',
                        borderRadius: 'var(--radius-sm)',
                        marginBottom: 2,
                      }}
                      onClick={() => setExpandedType(expandedType === type ? null : type)}
                    >
                      <span>{type} ({findings.length})</span>
                      <span style={{ color: 'var(--text-muted)' }}>
                        {expandedType === type ? '\u25BC' : '\u25B6'}
                      </span>
                    </div>
                    {expandedType === type && findings.map((finding, i) => (
                      <div key={i} style={findingItemStyle}>
                        <div style={{ flex: 1 }}>
                          <div style={{ fontWeight: 500, marginBottom: 2 }}>{finding.path}</div>
                          <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                            {formatBytes(finding.size)} - {finding.description}
                          </div>
                        </div>
                        {getRiskBadge(finding.riskLevel)}
                      </div>
                    ))}
                  </div>
                ))
              )}
            </div>
          </>
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
