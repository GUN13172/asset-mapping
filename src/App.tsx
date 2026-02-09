import React, { useState, useMemo } from 'react';
import { Layout, Menu, theme, ConfigProvider } from 'antd';
import { useTheme } from './hooks/useTheme';
import {
  SearchOutlined,
  KeyOutlined,
  SettingOutlined,
  ExportOutlined,
  HistoryOutlined,
  SwapOutlined,
  SendOutlined,
  DatabaseOutlined,
  BugOutlined
} from '@ant-design/icons';
import AssetQuery from './components/AssetQuery';
import ApiKeyManagement from './components/ApiKeyManagement';
import Settings from './components/Settings';
import ExportData from './components/ExportData';
import HistoryRecords from './components/HistoryRecords';
import QueryConverter from './components/QueryConverter';
import Resender from './components/Resender';
import PocManager from './components/PocManager';
import VulnerabilityScan from './components/VulnerabilityScan';
import 'antd/dist/reset.css';

const { Header, Content, Sider } = Layout;

type MenuItem = {
  key: string;
  icon: React.ReactNode;
  label: string;
};

const App: React.FC = () => {
  const [selectedKey, setSelectedKey] = useState('asset-query');
  const { theme: currentTheme } = useTheme();

  // 根据当前主题决定使用暗色还是亮色算法
  const isDark = useMemo(() => {
    if (currentTheme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return currentTheme === 'dark';
  }, [currentTheme]);

  const menuItems: MenuItem[] = [
    {
      key: 'asset-query',
      icon: <SearchOutlined />,
      label: '资产测绘'
    },
    {
      key: 'poc-manager',
      icon: <DatabaseOutlined />,
      label: 'POC管理'
    },
    {
      key: 'vulnerability-scan',
      icon: <BugOutlined />,
      label: '漏洞扫描'
    },
    {
      key: 'resender',
      icon: <SendOutlined />,
      label: '重发器'
    },
    {
      key: 'query-converter',
      icon: <SwapOutlined />,
      label: '语法转换'
    },
    {
      key: 'api-keys',
      icon: <KeyOutlined />,
      label: 'API密钥管理'
    },
    {
      key: 'export',
      icon: <ExportOutlined />,
      label: '数据导出'
    },
    {
      key: 'history',
      icon: <HistoryOutlined />,
      label: '历史记录'
    },
    {
      key: 'settings',
      icon: <SettingOutlined />,
      label: '设置'
    }
  ];

  const handleMenuClick = (key: string) => {
    setSelectedKey(key);
  };

  // 使用条件渲染而不是动态组件，保持组件实例
  const renderContent = () => {
    return (
      <div className="fade-in">
        <div style={{ display: selectedKey === 'asset-query' ? 'block' : 'none' }}>
          <AssetQuery />
        </div>
        <div style={{ display: selectedKey === 'poc-manager' ? 'block' : 'none' }}>
          <PocManager />
        </div>
        <div style={{ display: selectedKey === 'vulnerability-scan' ? 'block' : 'none' }}>
          <VulnerabilityScan active={selectedKey === 'vulnerability-scan'} />
        </div>
        <div style={{ display: selectedKey === 'resender' ? 'block' : 'none' }}>
          <Resender />
        </div>
        <div style={{ display: selectedKey === 'api-keys' ? 'block' : 'none' }}>
          <ApiKeyManagement />
        </div>
        <div style={{ display: selectedKey === 'export' ? 'block' : 'none' }}>
          <ExportData />
        </div>
        <div style={{ display: selectedKey === 'query-converter' ? 'block' : 'none' }}>
          <QueryConverter />
        </div>
        <div style={{ display: selectedKey === 'history' ? 'block' : 'none' }}>
          <HistoryRecords />
        </div>
        <div style={{ display: selectedKey === 'settings' ? 'block' : 'none' }}>
          <Settings />
        </div>
      </div>
    );
  };

  return (
    <ConfigProvider
      theme={{
        algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
        token: isDark ? {
          colorPrimary: '#00d4ff',
          borderRadius: 8,
          colorBgContainer: '#16213e',
          colorBgLayout: '#0f0f23',
        } : {
          colorPrimary: '#1677ff',
          borderRadius: 8,
        },
      }}
    >
      <Layout style={{ minHeight: '100vh', background: 'var(--bg-primary)' }}>
        <Header style={{
          display: 'flex',
          alignItems: 'center',
          padding: '0 24px',
          background: 'var(--bg-secondary)',
          borderBottom: '1px solid var(--border-color)',
          height: '64px',
          zIndex: 10,
          backdropFilter: 'blur(12px)',
        }}>
          <div className="app-header-title" style={{ fontSize: '20px', fontWeight: 'bold', color: isDark ? '#00d4ff' : '#1677ff' }}>
            资产测绘与漏洞研究平台
          </div>
        </Header>
        <Layout className="site-layout">
          <Sider
            width={240}
            style={{
              background: 'var(--bg-secondary)',
              borderRight: '1px solid var(--border-color)',
              backdropFilter: 'blur(12px)',
            }}
          >
            <Menu
              mode="inline"
              selectedKeys={[selectedKey]}
              style={{
                height: '100%',
                borderRight: 0,
                background: 'transparent',
                paddingTop: '1rem'
              }}
              items={menuItems.map(item => ({
                key: item.key,
                icon: item.icon,
                label: item.label,
                onClick: () => handleMenuClick(item.key)
              }))}
            />
          </Sider>
          <Layout style={{ padding: '24px', background: 'transparent' }}>
            <Content
              style={{
                padding: 0,
                margin: 0,
                minHeight: 280,
                background: 'rgba(255, 255, 255, 0.02)',
                backdropFilter: 'blur(15px)',
                borderRadius: 12,
                border: '1px solid var(--border-color)',
                overflow: 'auto'
              }}
            >
              <div style={{ padding: '24px' }}>
                {renderContent()}
              </div>
            </Content>
          </Layout>
        </Layout>
      </Layout>
    </ConfigProvider>
  );
};

export default App;
