import { useEffect, useState } from 'react';

interface ProgressBarProps {
  value: number;
  label?: string;
  statusText?: string;
  variant?: 'determinate' | 'indeterminate';
  color?: string;
}

export default function ProgressBar({ value, label, statusText, variant = 'determinate', color }: ProgressBarProps) {
  const [displayValue, setDisplayValue] = useState(0);
  const barColor = color || 'var(--accent)';
  const isIndeterminate = variant === 'indeterminate';

  useEffect(() => {
    if (isIndeterminate) return;
    const timer = setTimeout(() => setDisplayValue(value), 50);
    return () => clearTimeout(timer);
  }, [value, isIndeterminate]);

  useEffect(() => {
    if (!isIndeterminate) return;
    const interval = setInterval(() => {
      setDisplayValue((prev) => (prev >= 90 ? 10 : prev + 10));
    }, 400);
    return () => clearInterval(interval);
  }, [isIndeterminate]);

  const containerStyle: React.CSSProperties = {
    width: '100%',
    marginBottom: 12,
  };

  const labelRowStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    marginBottom: 6,
  };

  const labelStyle: React.CSSProperties = {
    fontSize: '0.85rem',
    fontWeight: 500,
    color: 'var(--text-primary)',
  };

  const percentStyle: React.CSSProperties = {
    fontSize: '0.85rem',
    fontWeight: 600,
    color: barColor,
  };

  const trackStyle: React.CSSProperties = {
    width: '100%',
    height: 8,
    background: 'var(--progress-bg)',
    borderRadius: 4,
    overflow: 'hidden',
  };

  const fillStyle: React.CSSProperties = {
    height: '100%',
    width: isIndeterminate ? `${displayValue}%` : `${Math.min(Math.max(displayValue, 0), 100)}%`,
    background: barColor,
    borderRadius: 4,
    transition: isIndeterminate ? 'width 0.3s ease' : 'width 0.4s ease',
    position: 'relative',
  };

  const statusStyle: React.CSSProperties = {
    fontSize: '0.78rem',
    color: 'var(--text-muted)',
    marginTop: 6,
  };

  return (
    <div style={containerStyle}>
      {(label || variant === 'determinate') && (
        <div style={labelRowStyle}>
          {label && <span style={labelStyle}>{label}</span>}
          {variant === 'determinate' && <span style={percentStyle}>{Math.round(value)}%</span>}
        </div>
      )}
      <div style={trackStyle}>
        <div style={fillStyle} />
      </div>
      {statusText && <div style={statusStyle}>{statusText}</div>}
    </div>
  );
}
