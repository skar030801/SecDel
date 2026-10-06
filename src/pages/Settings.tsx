import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Settings as SettingsType } from '../types';

export default function Settings() {
  const [settings, setSettings] = useState<SettingsType>({
    theme: 'dark',
    defaultSecurityLevel: 2,
    overwritePasses: 3,
    verifyAfterDelete: true,
    trimAfterDelete: true,
    autoGenerateReport: false,
    notificationLevel: 'info',
    adminMode: false,
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    invoke<SettingsType>('get_settings')
      .then((result) => {
        if (!cancelled) setSettings(result);
      })
      .catch(() => {
        // Keep the defaults when settings cannot be loaded.
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function saveSettings() {
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      await invoke('save_settings', { settings });
      setSuccess('Settings saved successfully');

      const body = document.body;
      if (settings.theme === 'dark') {
        body.classList.add('dark');
      } else {
        body.classList.remove('dark');
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  async function restoreDefaults() {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SettingsType>('get_default_settings');
      setSettings(result);
      setSuccess('Defaults restored (not saved yet)');
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }

  function update<K extends keyof SettingsType>(key: K, value: SettingsType[K]) {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }

  const cardStyle: React.CSSProperties = {
    background: 'var(--card-bg)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: 20,
    boxShadow: 'var(--shadow)',
    marginBottom: 16,
  };

  const rowStyle: React.CSSProperties = {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 0',
    borderBottom: '1px solid var(--border)',
  };

  const aboutSectionStyle: React.CSSProperties = {
    ...cardStyle,
    fontSize: '0.85rem',
    lineHeight: 1.8,
  };

  if (loading) {
    return (
      <div>
        <h1 className="page-title">Settings</h1>
        <div style={{ display: 'flex', justifyContent: 'center', padding: 48 }}>
          <div className="spinner" />
        </div>
      </div>
    );
  }

  return (
    <div className="fade-in">
      <h1 className="page-title">Settings</h1>
      <p className="page-subtitle">Configure application preferences</p>

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">{success}</div>}

      <div style={{ maxWidth: 700 }}>
        <div style={cardStyle}>
          <h3 className="card-header">Appearance</h3>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Theme</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Choose dark or light mode</div>
            </div>
            <select
              className="select"
              style={{ width: 140 }}
              value={settings.theme}
              onChange={(e) => update('theme', e.target.value)}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>
        </div>

        <div style={cardStyle}>
          <h3 className="card-header">Security</h3>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Default Security Level</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Default overwrite pattern level</div>
            </div>
            <select
              className="select"
              style={{ width: 140 }}
              value={settings.defaultSecurityLevel}
              onChange={(e) => update('defaultSecurityLevel', Number(e.target.value))}
            >
              {[1, 2, 3, 4].map((v) => (
                <option key={v} value={v}>Level {v}</option>
              ))}
            </select>
          </div>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Overwrite Passes</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Number of overwrite passes</div>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
              <input
                type="range"
                min={1}
                max={35}
                value={settings.overwritePasses}
                onChange={(e) => update('overwritePasses', Number(e.target.value))}
                style={{ accentColor: 'var(--accent)', width: 120 }}
              />
              <span style={{ fontSize: '0.85rem', fontWeight: 500, minWidth: 30 }}>{settings.overwritePasses}</span>
            </div>
          </div>
        </div>

        <div style={cardStyle}>
          <h3 className="card-header">Operations</h3>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Verify After Delete</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Always verify files after deletion</div>
            </div>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.verifyAfterDelete}
                onChange={(e) => update('verifyAfterDelete', e.target.checked)}
              />
            </label>
          </div>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>TRIM After Delete</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Execute TRIM after each deletion</div>
            </div>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.trimAfterDelete}
                onChange={(e) => update('trimAfterDelete', e.target.checked)}
              />
            </label>
          </div>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Auto-Generate Report</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Generate report after each operation</div>
            </div>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.autoGenerateReport}
                onChange={(e) => update('autoGenerateReport', e.target.checked)}
              />
            </label>
          </div>
        </div>

        <div style={cardStyle}>
          <h3 className="card-header">Notifications</h3>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Notification Level</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Minimum severity for notifications</div>
            </div>
            <select
              className="select"
              style={{ width: 140 }}
              value={settings.notificationLevel}
              onChange={(e) => update('notificationLevel', e.target.value)}
            >
              <option value="debug">Debug</option>
              <option value="info">Info</option>
              <option value="warn">Warning</option>
              <option value="error">Error</option>
            </select>
          </div>
        </div>

        <div style={cardStyle}>
          <h3 className="card-header">Advanced</h3>
          <div style={rowStyle}>
            <div>
              <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>Admin Mode</div>
              <div style={{ fontSize: '0.78rem', color: 'var(--text-muted)' }}>Enable advanced operations (requires restart)</div>
            </div>
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={settings.adminMode}
                onChange={(e) => update('adminMode', e.target.checked)}
              />
            </label>
          </div>
        </div>

        <div style={{ display: 'flex', gap: 12, marginBottom: 24 }}>
          <button className="btn btn-primary" onClick={saveSettings} disabled={saving}>
            {saving ? (
              <><span className="spinner" style={{ width: 16, height: 16 }} /> Saving...</>
            ) : (
              'Save Settings'
            )}
          </button>
          <button className="btn btn-secondary" onClick={restoreDefaults}>
            Restore Defaults
          </button>
        </div>

        <div style={aboutSectionStyle}>
          <h3 className="card-header">About</h3>
          <div style={rowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Application</span>
            <span style={{ fontWeight: 500 }}>SecDel - Secure Deletion</span>
          </div>
          <div style={rowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Version</span>
            <span style={{ fontWeight: 500 }}>1.0.0</span>
          </div>
          <div style={rowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Framework</span>
            <span style={{ fontWeight: 500 }}>Tauri v2 + React</span>
          </div>
          <div style={rowStyle}>
            <span style={{ color: 'var(--text-secondary)' }}>Description</span>
            <span style={{ fontWeight: 500, textAlign: 'right', maxWidth: '60%' }}>
              Secure file deletion and storage sanitization tool
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}
