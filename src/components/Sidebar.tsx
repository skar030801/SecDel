import type { Page } from '../types';

interface SidebarProps {
  activePage: Page;
  onNavigate: (page: Page) => void;
  isDark: boolean;
  onToggleTheme: () => void;
}

interface NavItem {
  page: Page;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { page: 'dashboard', label: 'Dashboard', icon: '\u{1F4CA}' },
  { page: 'devices', label: 'Devices', icon: '\u{1F4BE}' },
  { page: 'secure-delete', label: 'Secure Delete', icon: '\u{1F5D1}\uFE0F' },
  { page: 'full-drive-erase', label: 'Full Drive Erase', icon: '\u26A1' },
  { page: 'artifact-scanner', label: 'Scanner', icon: '\u{1F50D}' },
  { page: 'verification', label: 'Verification', icon: '\u2714\uFE0F' },
  { page: 'reports', label: 'Reports', icon: '\u{1F4CB}' },
  { page: 'settings', label: 'Settings', icon: '\u2699\uFE0F' },
];

export default function Sidebar({ activePage, onNavigate, isDark, onToggleTheme }: SidebarProps) {
  const sidebarStyle: React.CSSProperties = {
    width: 220,
    minWidth: 220,
    height: '100vh',
    background: 'var(--sidebar-bg)',
    borderRight: '1px solid var(--border)',
    display: 'flex',
    flexDirection: 'column',
    transition: 'background-color 0.3s ease, border-color 0.3s ease',
  };

  const logoStyle: React.CSSProperties = {
    padding: '24px 20px 20px',
    borderBottom: '1px solid var(--border)',
  };

  const logoTextStyle: React.CSSProperties = {
    fontSize: '1.5rem',
    fontWeight: 700,
    color: 'var(--accent)',
    letterSpacing: '-0.5px',
  };

  const navStyle: React.CSSProperties = {
    flex: 1,
    padding: '12px 8px',
    overflowY: 'auto',
  };

  const getLinkStyle = (isActive: boolean): React.CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: 10,
    padding: '10px 14px',
    borderRadius: 'var(--radius)',
    fontSize: '0.875rem',
    fontWeight: isActive ? 600 : 400,
    color: isActive ? 'var(--accent)' : 'var(--sidebar-text)',
    background: isActive ? 'var(--sidebar-active)' : 'transparent',
    cursor: 'pointer',
    transition: 'all 0.2s ease',
    marginBottom: 2,
    border: 'none',
    width: '100%',
    textAlign: 'left',
  });

  const bottomStyle: React.CSSProperties = {
    padding: '16px 20px',
    borderTop: '1px solid var(--border)',
  };

  const toggleBtnStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    padding: '8px 14px',
    borderRadius: 'var(--radius)',
    background: 'var(--bg-tertiary)',
    color: 'var(--text-primary)',
    fontSize: '0.8rem',
    cursor: 'pointer',
    border: '1px solid var(--border)',
    width: '100%',
    justifyContent: 'center',
    transition: 'all 0.2s ease',
  };

  const versionStyle: React.CSSProperties = {
    fontSize: '0.7rem',
    color: 'var(--text-muted)',
    textAlign: 'center',
    marginTop: 8,
  };

  return (
    <div style={sidebarStyle}>
      <div style={logoStyle}>
        <div style={logoTextStyle}>SecDel</div>
        <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', marginTop: 2 }}>Secure Deletion</div>
      </div>
      <nav style={navStyle}>
        {navItems.map((item) => (
          <button
            key={item.page}
            style={getLinkStyle(activePage === item.page)}
            onClick={() => onNavigate(item.page)}
            onMouseEnter={(e) => {
              if (activePage !== item.page) {
                e.currentTarget.style.background = 'var(--sidebar-hover)';
              }
            }}
            onMouseLeave={(e) => {
              if (activePage !== item.page) {
                e.currentTarget.style.background = 'transparent';
              }
            }}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </button>
        ))}
      </nav>
      <div style={bottomStyle}>
        <button style={toggleBtnStyle} onClick={onToggleTheme}>
          {isDark ? '\u2600\uFE0F Light Mode' : '\u{1F319} Dark Mode'}
        </button>
        <div style={versionStyle}>v0.1.0</div>
      </div>
    </div>
  );
}
