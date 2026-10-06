import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ReportData } from '../types';

interface SectionOption {
  key: string;
  label: string;
}

const reportSections: SectionOption[] = [
  { key: 'summary', label: 'Summary' },
  { key: 'devices', label: 'Device Details' },
  { key: 'operations', label: 'Operations Log' },
  { key: 'verification', label: 'Verification Results' },
  { key: 'errors', label: 'Errors & Warnings' },
];

export default function Reports() {
  const [reportType, setReportType] = useState('json');
  const [selectedSections, setSelectedSections] = useState<string[]>(['summary']);
  const [generating, setGenerating] = useState(false);
  const [report, setReport] = useState<ReportData | null>(null);
  const [history, setHistory] = useState<ReportData[]>([]);
  const [error, setError] = useState<string | null>(null);

  const toggleSection = (key: string) => {
    setSelectedSections((prev) =>
      prev.includes(key) ? prev.filter((s) => s !== key) : [...prev, key]
    );
  };

  const generateReport = async () => {
    if (selectedSections.length === 0) return;
    setGenerating(true);
    setError(null);
    setReport(null);
    try {
      const result = await invoke<ReportData>('generate_report', {
        reportType,
        sections: selectedSections,
      });
      setReport(result);
      setHistory((prev) => [result, ...prev].slice(0, 20));
    } catch (err) {
      setError(String(err));
    } finally {
      setGenerating(false);
    }
  };

  const downloadReport = async () => {
    if (!report) return;
    try {
      await invoke('download_report', { reportId: report.id });
    } catch (err) {
      setError(String(err));
    }
  };

  const openReport = async () => {
    if (!report) return;
    try {
      await invoke('open_report', { reportId: report.id });
    } catch (err) {
      setError(String(err));
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

  const previewStyle: React.CSSProperties = {
    background: 'var(--bg-tertiary)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 16,
    fontSize: '0.78rem',
    fontFamily: "'Consolas', 'Courier New', monospace",
    maxHeight: 300,
    overflowY: 'auto',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-all',
    lineHeight: 1.5,
    color: 'var(--text-primary)',
  };

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
      <h1 className="page-title">Reports</h1>
      <p className="page-subtitle">Generate and manage reports</p>

      {error && <div className="error-message">{error}</div>}

      <div style={{ maxWidth: 700 }}>
        <div style={cardStyle}>
          <div className="form-group">
            <label className="form-label">Report Type</label>
            <select
              className="select"
              value={reportType}
              onChange={(e) => setReportType(e.target.value)}
              disabled={generating}
            >
              <option value="json">JSON</option>
              <option value="csv">CSV</option>
              <option value="pdf">PDF</option>
            </select>
          </div>
          <div className="form-group">
            <label className="form-label">Include Sections</label>
            <div className="checkbox-group">
              {reportSections.map((section) => (
                <label key={section.key} className="checkbox-label">
                  <input
                    type="checkbox"
                    checked={selectedSections.includes(section.key)}
                    onChange={() => toggleSection(section.key)}
                    disabled={generating}
                  />
                  {section.label}
                </label>
              ))}
            </div>
          </div>
          <button
            className="btn btn-primary"
            onClick={generateReport}
            disabled={generating || selectedSections.length === 0}
          >
            {generating ? (
              <><span className="spinner" style={{ width: 16, height: 16 }} /> Generating...</>
            ) : (
              <>{'\u{1F4CB}'} Generate Report</>
            )}
          </button>
        </div>

        {report && (
          <div style={cardStyle}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 16 }}>
              <h3 style={{ fontSize: '0.95rem', fontWeight: 600 }}>
                Report - {report.reportType.toUpperCase()}
              </h3>
              <div className="flex-row">
                <button className="btn btn-sm btn-secondary" onClick={downloadReport}>
                  Download
                </button>
                <button className="btn btn-sm btn-secondary" onClick={openReport}>
                  Open
                </button>
              </div>
            </div>
            <div style={{ display: 'flex', gap: 16, marginBottom: 16, fontSize: '0.82rem', color: 'var(--text-secondary)' }}>
              <span>Generated: {new Date(report.generatedAt).toLocaleString()}</span>
              <span>Size: {formatBytes(report.fileSize)}</span>
              <span>Sections: {report.sections.join(', ')}</span>
            </div>
            <div style={previewStyle}>
              {reportType === 'json' ? formatJsonPreview(report.data) : report.data}
            </div>
          </div>
        )}

        {history.length > 0 && (
          <div style={cardStyle}>
            <h3 style={{ fontSize: '0.95rem', fontWeight: 600, marginBottom: 16 }}>Report History</h3>
            {history.map((item, i) => (
              <div key={i} style={historyItemStyle}>
                <div>
                  <div style={{ fontWeight: 500 }}>{item.reportType.toUpperCase()} Report</div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
                    {new Date(item.generatedAt).toLocaleString()} - {formatBytes(item.fileSize)}
                  </div>
                </div>
                <div className="flex-row">
                  <button className="btn btn-sm btn-secondary" onClick={() => setReport(item)}>
                    View
                  </button>
                </div>
              </div>
            ))}
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

function formatJsonPreview(data: string): string {
  try {
    return JSON.stringify(JSON.parse(data), null, 2);
  } catch {
    return data;
  }
}
