import type { ReactNode } from 'react';
import Sidebar from './Sidebar';
import type { Page } from '../types';

interface LayoutProps {
  activePage: Page;
  onNavigate: (page: Page) => void;
  isDark: boolean;
  onToggleTheme: () => void;
  children: ReactNode;
}

export default function Layout({ activePage, onNavigate, isDark, onToggleTheme, children }: LayoutProps) {
  return (
    <div className="app-container">
      <Sidebar
        activePage={activePage}
        onNavigate={onNavigate}
        isDark={isDark}
        onToggleTheme={onToggleTheme}
      />
      <main className="main-content">
        {children}
      </main>
    </div>
  );
}
