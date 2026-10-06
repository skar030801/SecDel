import { useEffect, useRef } from 'react';

interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

interface LogViewerProps {
  entries: LogEntry[];
  maxEntries?: number;
}

const levelColors: Record<string, string> = {
  info: 'var(--log-info)',
  warn: 'var(--log-warn)',
  warning: 'var(--log-warn)',
  error: 'var(--log-error)',
  err: 'var(--log-error)',
  success: 'var(--log-success)',
  debug: 'var(--text-muted)',
};

export default function LogViewer({ entries, maxEntries = 500 }: LogViewerProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const displayed = entries.slice(-maxEntries);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [displayed.length]);

  const containerStyle: React.CSSProperties = {
    background: 'var(--bg-tertiary)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 12,
    maxHeight: 300,
    overflowY: 'auto',
    fontFamily: "'Consolas', 'Courier New', monospace",
    fontSize: '0.78rem',
    lineHeight: 1.6,
  };

  const lineStyle = (level: string): React.CSSProperties => ({
    color: levelColors[level] || 'var(--text-primary)',
    padding: '2px 0',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-all',
  });

  if (entries.length === 0) {
    const emptyStyle: React.CSSProperties = {
      ...containerStyle,
      color: 'var(--text-muted)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      minHeight: 80,
    };
    return <div style={emptyStyle}>No log entries yet.</div>;
  }

  return (
    <div style={{ position: 'relative' }}>
      <div ref={containerRef} style={containerStyle}>
        {displayed.map((entry, i) => (
          <div key={i} style={lineStyle(entry.level)}>
            <span style={{ color: 'var(--text-muted)', marginRight: 8 }}>{entry.timestamp}</span>
            <span style={{ fontWeight: 600, textTransform: 'uppercase', fontSize: '0.7rem', marginRight: 8 }}>
              [{entry.level}]
            </span>
            {entry.message}
          </div>
        ))}
      </div>
    </div>
  );
}
