import React, { Suspense, lazy, startTransition, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { theme, ConfigProvider, Tooltip, Spin, Space } from 'antd';
import { useTheme } from './hooks/useTheme';
import { endPerf, isPerfEnabled, PerfToken, scheduleIdle, startPerf } from './utils/perf';
import {
  SearchOutlined,
  KeyOutlined,
  SettingOutlined,
  ExportOutlined,
  HistoryOutlined,
  SwapOutlined,
  SendOutlined,
  DatabaseOutlined,
  BugOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  SunOutlined,
  MoonOutlined,
} from '@ant-design/icons';
import 'antd/dist/reset.css';

type ViewKey =
  | 'asset-query'
  | 'poc-manager'
  | 'vulnerability-scan'
  | 'resender'
  | 'query-converter'
  | 'api-keys'
  | 'export'
  | 'history'
  | 'settings';

type ViewModule = { default: React.ComponentType<any> };

const viewLoaders: Record<ViewKey, () => Promise<ViewModule>> = {
  'asset-query': () => import('./components/AssetQuery'),
  'poc-manager': () => import('./components/PocManager'),
  'vulnerability-scan': () => import('./components/VulnerabilityScan'),
  'resender': () => import('./components/Resender'),
  'query-converter': () => import('./components/QueryConverter'),
  'api-keys': () => import('./components/ApiKeyManagement'),
  'export': () => import('./components/ExportData'),
  'history': () => import('./components/HistoryRecords'),
  'settings': () => import('./components/Settings'),
};

const preloadCache = new Map<ViewKey, Promise<ViewModule>>();

const loadView = (key: ViewKey) => {
  const cached = preloadCache.get(key);
  if (cached) return cached;

  const loadPerfToken = startPerf('view-chunk-load', { view: key });
  const pending = viewLoaders[key]()
    .then((module) => {
      endPerf(loadPerfToken, { view: key, status: 'loaded' });
      return module;
    })
    .catch((error) => {
      preloadCache.delete(key);
      endPerf(loadPerfToken, {
        view: key,
        status: 'error',
        error: error instanceof Error ? error.message : String(error),
      });
      throw error;
    });
  preloadCache.set(key, pending);
  return pending;
};

const preloadView = (key: ViewKey) => { void loadView(key); };

const AssetQuery = lazy(() => loadView('asset-query'));
const ApiKeyManagement = lazy(() => loadView('api-keys'));
const Settings = lazy(() => loadView('settings'));
const ExportData = lazy(() => loadView('export'));
const HistoryRecords = lazy(() => loadView('history'));
const QueryConverter = lazy(() => loadView('query-converter'));
const Resender = lazy(() => loadView('resender'));
const PocManager = lazy(() => loadView('poc-manager'));
const VulnerabilityScan = lazy(() => loadView('vulnerability-scan'));
const PerfPanel = lazy(() => import('./components/PerfPanel'));

type NavItem = {
  key: ViewKey;
  icon: React.ReactNode;
  label: string;
};

const navItems: NavItem[] = [
  { key: 'asset-query', icon: <SearchOutlined />, label: '资产测绘' },
  { key: 'poc-manager', icon: <DatabaseOutlined />, label: 'POC管理' },
  { key: 'vulnerability-scan', icon: <BugOutlined />, label: '漏洞扫描' },
  { key: 'resender', icon: <SendOutlined />, label: '重发器' },
  { key: 'query-converter', icon: <SwapOutlined />, label: '语法转换' },
  { key: 'api-keys', icon: <KeyOutlined />, label: 'API密钥' },
  { key: 'export', icon: <ExportOutlined />, label: '数据导出' },
  { key: 'history', icon: <HistoryOutlined />, label: '历史记录' },
  { key: 'settings', icon: <SettingOutlined />, label: '设置' },
];

const likelyNextViews: Record<ViewKey, ViewKey[]> = {
  'asset-query': ['export', 'history', 'resender'],
  'poc-manager': ['vulnerability-scan'],
  'vulnerability-scan': ['poc-manager', 'history'],
  'resender': ['asset-query'],
  'query-converter': ['asset-query', 'export'],
  'api-keys': ['settings', 'asset-query'],
  'export': ['history', 'query-converter'],
  'history': ['export', 'asset-query'],
  'settings': ['api-keys'],
};

type AppProps = { bootstrapPerfToken?: PerfToken };

const App: React.FC<AppProps> = ({ bootstrapPerfToken = null }) => {
  const [selectedKey, setSelectedKey] = useState<ViewKey>('asset-query');
  const [collapsed, setCollapsed] = useState(false);
  const [visitedViews, setVisitedViews] = useState<Record<ViewKey, boolean>>({
    'asset-query': true,
    'poc-manager': false,
    'vulnerability-scan': false,
    'resender': false,
    'query-converter': false,
    'api-keys': false,
    'export': false,
    'history': false,
    'settings': false,
  });
  const activationPerfRef = useRef<Partial<Record<ViewKey, PerfToken>>>({});
  const perfPanelEnabled = useMemo(() => isPerfEnabled(), []);
  const { theme: currentTheme, setTheme } = useTheme();

  const isDark = useMemo(() => {
    if (currentTheme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return currentTheme === 'dark';
  }, [currentTheme]);

  const toggleTheme = () => setTheme(isDark ? 'light' : 'dark');

  const handleNavClick = (key: ViewKey) => {
    if (key !== selectedKey) {
      activationPerfRef.current[key] = startPerf('view-activate', { view: key, source: 'nav-click' });
    }
    preloadView(key);
    startTransition(() => {
      setSelectedKey(key);
      setVisitedViews((prev) => (prev[key] ? prev : { ...prev, [key]: true }));
    });
  };

  // Predictive preload on idle
  useEffect(() => {
    const dispose = scheduleIdle(() => {
      preloadView('query-converter');
      preloadView('api-keys');
    }, 1200);
    return dispose;
  }, []);

  useEffect(() => {
    const dispose = scheduleIdle(() => {
      likelyNextViews[selectedKey].forEach(preloadView);
    }, 600);
    return dispose;
  }, [selectedKey]);

  useEffect(() => {
    const frame = window.requestAnimationFrame(() => {
      endPerf(bootstrapPerfToken, { stage: 'shell-visible', initialView: selectedKey });
    });
    return () => window.cancelAnimationFrame(frame);
  }, [bootstrapPerfToken, selectedKey]);

  const handleViewVisible = useCallback((key: ViewKey) => {
    const token = activationPerfRef.current[key];
    if (!token) return;
    endPerf(token, { view: key, status: 'visible' });
    delete activationPerfRef.current[key];
  }, []);

  const renderLazyView = (key: ViewKey, node: React.ReactNode) => {
    if (!visitedViews[key]) return null;
    return (
      <div style={{ display: selectedKey === key ? 'block' : 'none' }}>
        <Suspense
          fallback={
            <div style={{ display: 'flex', justifyContent: 'center', padding: '64px 0' }}>
              <Space direction="vertical" align="center" size={12}>
                <Spin size="large" />
                <span style={{ color: 'var(--text-secondary)' }}>加载中...</span>
              </Space>
            </div>
          }
        >
          <ViewRenderProbe viewKey={key} active={selectedKey === key} onVisible={handleViewVisible}>
            {node}
          </ViewRenderProbe>
        </Suspense>
      </div>
    );
  };

  return (
    <ConfigProvider
      theme={{
        algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: {
          borderRadius: 6,
          colorPrimary: '#d97706',
          fontFamily: `-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', Roboto, sans-serif`,
          ...(isDark ? {
            colorBgContainer: '#1c1917',
            colorBgElevated: '#292524',
            colorBgLayout: '#141210',
            colorBorderSecondary: 'rgba(255, 200, 120, 0.08)',
            colorText: '#faf5ef',
            colorTextSecondary: '#a89a8c',
          } : {
            colorBgContainer: '#ffffff',
            colorBgElevated: '#ffffff',
            colorBgLayout: '#fdfcfa',
            colorBorderSecondary: 'rgba(28, 25, 23, 0.08)',
            colorText: '#1c1917',
            colorTextSecondary: '#57534e',
          }),
        },
        components: {
          Table: { colorBgContainer: 'transparent', headerBg: 'transparent' },
          Card: { colorBorderSecondary: isDark ? 'rgba(255, 200, 120, 0.08)' : 'rgba(28, 25, 23, 0.08)' },
        },
      }}
    >
      <div className="app-shell-root">
        {/* Sidebar */}
        <aside className={`app-sidebar${collapsed ? ' is-collapsed' : ''}`}>
          <div className="sidebar-brand">
            <div className="sidebar-brand-icon">A</div>
            <div className="sidebar-brand-text">
              <div className="sidebar-brand-title">资产测绘平台</div>
              <div className="sidebar-brand-subtitle">Asset Mapping</div>
            </div>
          </div>

          <nav className="sidebar-nav">
            <div className="sidebar-section-label">功能</div>
            {navItems.map((item) => (
              <Tooltip key={item.key} title={collapsed ? item.label : ''} placement="right">
                <div
                  className={`sidebar-nav-item${selectedKey === item.key ? ' is-active' : ''}`}
                  onClick={() => handleNavClick(item.key)}
                  onMouseEnter={() => preloadView(item.key)}
                >
                  <span className="sidebar-nav-item-icon">{item.icon}</span>
                  <span>{item.label}</span>
                </div>
              </Tooltip>
            ))}
          </nav>

          <div className="sidebar-footer">
            <button className="sidebar-footer-btn" onClick={() => setCollapsed(!collapsed)}>
              {collapsed ? <MenuUnfoldOutlined /> : <MenuFoldOutlined />}
            </button>
            <span className="sidebar-version">v1.0.0</span>
            <button className="sidebar-footer-btn" onClick={toggleTheme}>
              {isDark ? <SunOutlined /> : <MoonOutlined />}
            </button>
          </div>
        </aside>

        {/* Main content */}
        <main className="app-main">
          <div className="app-content">
            <div className="app-content-inner fade-in">
              {renderLazyView('asset-query', <AssetQuery />)}
              {renderLazyView('poc-manager', <PocManager />)}
              {renderLazyView('vulnerability-scan', <VulnerabilityScan active={selectedKey === 'vulnerability-scan'} />)}
              {renderLazyView('resender', <Resender active={selectedKey === 'resender'} />)}
              {renderLazyView('api-keys', <ApiKeyManagement />)}
              {renderLazyView('export', <ExportData />)}
              {renderLazyView('query-converter', <QueryConverter />)}
              {renderLazyView('history', <HistoryRecords active={selectedKey === 'history'} />)}
              {renderLazyView('settings', <Settings />)}
            </div>
          </div>
        </main>
      </div>

      {perfPanelEnabled && (
        <Suspense fallback={null}>
          <PerfPanel />
        </Suspense>
      )}
    </ConfigProvider>
  );
};

// Helper component for perf tracking
type ViewRenderProbeProps = {
  viewKey: ViewKey;
  active: boolean;
  onVisible: (viewKey: ViewKey) => void;
  children: React.ReactNode;
};

const ViewRenderProbe: React.FC<ViewRenderProbeProps> = ({ viewKey, active, onVisible, children }) => {
  useEffect(() => {
    if (!active) return;
    const frame = window.requestAnimationFrame(() => onVisible(viewKey));
    return () => window.cancelAnimationFrame(frame);
  }, [active, onVisible, viewKey]);
  return <>{children}</>;
};

export default App;
