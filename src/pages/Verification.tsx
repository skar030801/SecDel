import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { VerificationResult } from '../types';

export default function Verification() {
  const [targetPath, setTargetPath] = useState('');
  const [expectedHash, setExpectedHash] = useState('');
  const [verifying, setVerifying] = useState(false);
  const [result, setResult] = useState<VerificationResult | null>(null);
  const [history, setHistory] = useState<VerificationResult[]>([]);
  const [error, setError] = useState<string | null>(null);

  const selectPath = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'All Files', extensions: ['*'] }],
      });
      if (selected) setTargetPath(selected);
    } catch (err) {
      setError(String(err));
    }
  };

  const startVerification = async () => {
    if (!targetPath) return;
    setVerifying(true);
    setError(null);
    setResult(null);
    try {
      const verificationResult = await invoke<VerificationResult>('verify_deletion', {
        path: targetPath,
        expectedHash: expectedHash || null,
      });
      setResult(verificationResult);
      setHistory((prev) => [verificationResult, ...prev].slice(0, 20));
    } catch (err) {
      setError(String(err));
    } finally {
      setVerifying(false);
    }
  };

  const cardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    marginBottom: 16,
  };

  const confidenceMeterStyle = (): React.CSSProperties => ({
    width: '100%',
    height: 24,
    background: 'var(--progress-bg)',
    borderRadius: 12,
    overflow: 'hidden',
    position: 'relative',
    marginTop: 8,
  });

  const confidenceFillStyle = (value: number): React.CSSProperties => ({
    height: '100%',
    width: `${value}%`,
    background: value >= 80 ? 'var(--success)' : value >= 50 ? 'var(--warning)' : 'var(--danger)',
    borderRadius: 12,
    transition: 'width 0.6s ease',
  });

  const historyItemStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '10px 12px',
    borderBottom: '1px solid var(--border)',
    fontSize: '0.82rem',
  };

  return (
    <div className="fade-in">
      <h1 className="page-title">Verification</h1>
      <p className="page-subtitle">Verify that files have been securely deleted</p>

      {error && <div className="error-message">{error}</div>}

      <div style={{ maxWidth: 700 }}>
        <div style={cardStyle}>
          <div className="form-group">
            <label className="form-label">Target Path</label>
            <div className="flex-row">
              <input
                className="input"
                type="text"
                value={targetPath}
                onChange={(e) => setTargetPath(e.target.value)}
                placeholder="Select a file or folder to verify..."
                disabled={verifying}
              />
              <button className="btn btn-secondary" onClick={selectPath} disabled={verifying}>
                Browse
              </button>
            </div>
          </div>
          <div className="form-group">
            <label className="form-label">Expected Hash (optional)</label>
            <input
              className="input"
              type="text"
              value={expectedHash}
              onChange={(e) => setExpectedHash(e.target.value)}
              placeholder="SHA-256 hash for comparison..."
              disabled={verifying}
            />
            <div className="form-hint">Leave empty to auto-detect and verify</div>
          </div>
          <button
            className="btn btn-primary"
            onClick={startVerification}
            disabled={verifying || !targetPath}
          >
            {verifying ? (
              <><span className="spinner" style={{ width: 16, height: 16 }} /> Verifying...</>
            ) : (
              <>{'\u2714\uFE0F'} Verify</>
            )}
          </button>
        </div>

        {result && (
          <div style={cardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Verification Result</h3>
            <div style={{ textAlign: 'center', marginBottom: 16 }}>
              <div style={{
                fontSize: '3rem',
                marginBottom: 8,
                color: result.match ? 'var(--success)' : 'var(--danger)',
              }}>
                {result.match ? '\u2714\uFE0F' : '\u274C'}
              </div>
              <div style={{
                fontSize: '1.1rem',
                fontWeight: 600,
                color: result.match ? 'var(--success)' : 'var(--danger)',
              }}>
                {result.match ? 'Verified - File Successfully Deleted' : 'Verification Failed'}
              </div>
            </div>
            <div style={confidenceMeterStyle()}>
              <div style={confidenceFillStyle(result.confidence)} />
            </div>
            <div style={{ textAlign: 'center', marginTop: 4, fontSize: '0.82rem', color: 'var(--text-muted)' }}>
              Confidence: {result.confidence}%
            </div>
            <div style={{ marginTop: 16, display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12, fontSize: '0.85rem' }}>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Method</div>
                <div style={{ fontWeight: 500 }}>{result.method}</div>
              </div>
              <div>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Time</div>
                <div style={{ fontWeight: 500 }}>{new Date(result.timestamp).toLocaleString()}</div>
              </div>
              <div style={{ gridColumn: '1 / -1' }}>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Target</div>
                <div style={{ fontWeight: 500, wordBreak: 'break-all' }}>{result.targetPath}</div>
              </div>
              {result.expectedHash && (
                <div style={{ gridColumn: '1 / -1' }}>
                  <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Expected Hash</div>
                  <div style={{ fontWeight: 500, fontFamily: 'monospace', fontSize: '0.78rem', wordBreak: 'break-all' }}>{result.expectedHash}</div>
                </div>
              )}
              <div style={{ gridColumn: '1 / -1' }}>
                <div style={{ color: 'var(--text-muted)', fontSize: '0.72rem', textTransform: 'uppercase' }}>Actual Hash</div>
                <div style={{ fontWeight: 500, fontFamily: 'monospace', fontSize: '0.78rem', wordBreak: 'break-all' }}>{result.actualHash}</div>
              </div>
            </div>
          </div>
        )}

        {history.length > 0 && (
          <div style={cardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Verification History</h3>
            {history.map((item, i) => (
              <div key={i} style={historyItemStyle}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ color: item.match ? 'var(--success)' : 'var(--danger)' }}>
                    {item.match ? '\u2714\uFE0F' : '\u274C'}
                  </span>
                  <div>
                    <div style={{ fontWeight: 500, fontSize: '0.82rem' }}>{item.targetPath.split(/[\\/]/).pop()}</div>
                    <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
                      {new Date(item.timestamp).toLocaleString()}
                    </div>
                  </div>
                </div>
                <span style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>
                  {item.confidence}%
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
