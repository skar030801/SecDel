import { useEffect, useState } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Page, ProgressEvent } from './types';
import Layout from './components/Layout';
import Dashboard from './pages/Dashboard';
import Devices from './pages/Devices';
import SecureDelete from './pages/SecureDelete';
import FullDriveErase from './pages/FullDriveErase';
import ArtifactScanner from './pages/ArtifactScanner';
import Verification from './pages/Verification';
import Reports from './pages/Reports';
import Settings from './pages/Settings';
import './App.css';

export default function App() {
  const [activePage, setActivePage] = useState<Page>('dashboard');
  const [isDark, setIsDark] = useState(true);

  useEffect(() => {
    const body = document.body;
    if (isDark) {
      body.classList.add('dark');
    } else {
      body.classList.remove('dark');
    }
  }, [isDark]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    const setupListener = async () => {
      unlisten = await listen<ProgressEvent>('progress', (event) => {
        console.log('Progress:', event.payload);
      });
    };
    setupListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const toggleTheme = () => {
    setIsDark((prev) => !prev);
  };

  const navigate = (page: Page) => {
    setActivePage(page);
  };

  const renderPage = () => {
    switch (activePage) {
      case 'dashboard':
        return <Dashboard onNavigate={(p) => setActivePage(p)} />;
      case 'devices':
        return <Devices />;
      case 'secure-delete':
        return <SecureDelete />;
      case 'full-drive-erase':
        return <FullDriveErase />;
      case 'artifact-scanner':
        return <ArtifactScanner />;
      case 'verification':
        return <Verification />;
      case 'reports':
        return <Reports />;
      case 'settings':
        return <Settings />;
      default:
        return <Dashboard onNavigate={(p) => setActivePage(p)} />;
    }
  };

  return (
    <Layout
      activePage={activePage}
      onNavigate={navigate}
      isDark={isDark}
      onToggleTheme={toggleTheme}
    >
      {renderPage()}
    </Layout>
  );
}
