import React, { useState, useCallback, useEffect, useRef } from 'react';
import { Card, Tabs, Input, Button, Table, Select, Space, Checkbox, Alert, AutoComplete, message } from 'antd';
import { SearchOutlined, DownloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import ProgressModal, { ProgressStatus, ProgressLog } from './ProgressModal';

interface AssetResult {
  url: string;
  ip: string;
  port: string;
  web_title?: string;
  country?: string;
  province?: string;
  city?: string;
  server?: string;
}

interface ProgressEventPayload {
  taskId: string;
  percent: number;
  status: string;
  statusText: string;
  logMessage?: string;
  logType?: string;
  currentPage?: number;
  totalPages?: number;
  totalResults?: number;
  fetchedResults?: number;
}

const AssetQuery: React.FC = () => {
  const [platform, setPlatform] = useState<string>('hunter');
  const [query, setQuery] = useState<string>('');
  const [loading, setLoading] = useState<boolean>(false);
  const [results, setResults] = useState<AssetResult[]>([]);
  const [totalResults, setTotalResults] = useState<number>(0);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [pageSize, setPageSize] = useState<number>(20);
  const [province, setProvince] = useState<string>('');
  const [city, setCity] = useState<string>('');
  const [appendToQuery, setAppendToQuery] = useState<boolean>(true);
  const [autoCompleteOptions, setAutoCompleteOptions] = useState<{ value: string; label: React.ReactNode }[]>([]);

  // 搜索进度弹窗状态
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [searchStatus, setSearchStatus] = useState<ProgressStatus>('idle');
  const [searchPercent, setSearchPercent] = useState(0);
  const [searchStatusText, setSearchStatusText] = useState('');
  const [searchLogs, setSearchLogs] = useState<ProgressLog[]>([]);

  // 导出进度弹窗状态
  const [exportModalOpen, setExportModalOpen] = useState(false);
  const [exportStatus, setExportStatus] = useState<ProgressStatus>('idle');
  const [exportPercent, setExportPercent] = useState(0);
  const [exportStatusText, setExportStatusText] = useState('');
  const [exportLogs, setExportLogs] = useState<ProgressLog[]>([]);
  const [exportSummary, setExportSummary] = useState<{ label: string; value: string | number }[]>([]);

  // 事件监听器引用
  const unlistenRef = useRef<UnlistenFn | null>(null);

  // 监听导出进度事件
  useEffect(() => {
    const setupListener = async () => {
      unlistenRef.current = await listen<ProgressEventPayload>('export-progress', (event) => {
        const data = event.payload;
        setExportPercent(data.percent);
        setExportStatus(data.status as ProgressStatus);
        setExportStatusText(data.statusText);

        if (data.logMessage) {
          const now = new Date().toLocaleTimeString();
          setExportLogs(prev => [...prev, {
            time: now,
            message: data.logMessage!,
            type: (data.logType || 'info') as ProgressLog['type'],
          }]);
        }

        // 更新摘要
        const summaryItems: { label: string; value: string | number }[] = [];
        if (data.totalPages != null) summaryItems.push({ label: '总页数', value: data.totalPages });
        if (data.currentPage != null) summaryItems.push({ label: '当前页', value: data.currentPage });
        if (data.totalResults != null) summaryItems.push({ label: '总结果数', value: data.totalResults });
        if (data.fetchedResults != null) summaryItems.push({ label: '已获取', value: data.fetchedResults });
        if (summaryItems.length > 0) setExportSummary(summaryItems);
      });
    };
    setupListener();

    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, []);

  // 平台查询语法提示
  const syntaxHelp = {
    hunter: [
      { label: 'domain.suffix="test.com"', description: '搜索域名后缀' },
      { label: 'ip="1.1.1.1"', description: '搜索IP' },
      { label: 'web.title="登录"', description: '搜索网页标题' },
      { label: 'header="thinkphp"', description: '搜索HTTP头' },
      { label: 'app.name="ThinkPHP"', description: '搜索应用框架' },
      { label: 'port="3306"', description: '搜索端口' },
      { label: 'status_code="200"', description: '搜索状态码' },
      { label: 'protocol="https"', description: '搜索协议' },
      { label: 'ip.province="北京市"', description: '搜索省份' },
      { label: 'ip.city="北京市"', description: '搜索城市' },
      { label: 'ip.country="中国"', description: '搜索国家' },
      { label: 'web.body="login"', description: '搜索网页内容' },
      { label: 'cert="baidu"', description: '搜索证书' },
      { label: 'banner="nginx"', description: '搜索Banner' },
    ],
    fofa: [
      { label: 'domain="test.com"', description: '搜索域名' },
      { label: 'ip="1.1.1.1"', description: '搜索IP' },
      { label: 'title="登录"', description: '搜索网页标题' },
      { label: 'header="nginx"', description: '搜索HTTP头' },
      { label: 'server=="Microsoft-IIS/10"', description: '搜索服务器' },
      { label: 'port="6379"', description: '搜索端口' },
      { label: 'protocol="https"', description: '搜索协议' },
      { label: 'country="CN"', description: '搜索国家' },
      { label: 'region="Beijing"', description: '搜索地区' },
      { label: 'city="Beijing"', description: '搜索城市' },
      { label: 'body="login"', description: '搜索网页内容' },
      { label: 'cert="baidu"', description: '搜索证书' },
      { label: 'banner="nginx"', description: '搜索Banner' },
    ],
    quake: [
      { label: 'domain: test.com', description: '搜索域名' },
      { label: 'ip: "1.1.1.1"', description: '搜索IP' },
      { label: 'title: "登录"', description: '搜索网页标题' },
      { label: 'response: "nginx"', description: '搜索响应内容' },
      { label: 'service: "IIS"', description: '搜索服务' },
      { label: 'port: 3389', description: '搜索端口' },
      { label: 'protocol: "https"', description: '搜索协议' },
      { label: 'country: "China"', description: '搜索国家' },
      { label: 'province: "Beijing"', description: '搜索省份' },
      { label: 'city: "Beijing"', description: '搜索城市' },
      { label: 'cert: "baidu"', description: '搜索证书' },
      { label: 'banner: "nginx"', description: '搜索Banner' },
    ],
    daydaymap: [
      { label: 'domain:"test.com"', description: '搜索域名' },
      { label: 'ip:"1.1.1.1"', description: '搜索IP地址' },
      { label: 'ip:"1.1.1.0/24"', description: '搜索IP段（CIDR）' },
      { label: 'title:"登录"', description: '搜索网页标题' },
      { label: 'server:"nginx"', description: '搜索服务器' },
      { label: 'app:"WordPress"', description: '搜索应用' },
      { label: 'port:"80"', description: '搜索端口' },
      { label: 'protocol:"https"', description: '搜索协议' },
      { label: 'country:"中国"', description: '搜索国家' },
      { label: 'province:"北京"', description: '搜索省份' },
      { label: 'city:"北京"', description: '搜索城市' },
      { label: 'body:"login"', description: '搜索网页内容' },
      { label: 'cert:"baidu"', description: '搜索证书' },
      { label: 'banner:"nginx"', description: '搜索Banner' },
    ],
  };

  // 查询占位符
  const placeholders = {
    hunter: '例如: domain.suffix="test.com" && ip.province="北京市"',
    fofa: '例如: domain="test.com" && country="CN"',
    quake: '例如: domain: test.com AND country: "China"',
    daydaymap: '例如: ip:"1.1.1.0/24" 或 domain:"test.com" (注意：使用冒号和引号)',
  };

  // 表格列定义
  const columns = [
    {
      title: 'URL',
      dataIndex: 'url',
      key: 'url',
      render: (text: string) => <a href={text} target="_blank" rel="noopener noreferrer">{text}</a>,
    },
    {
      title: 'IP',
      dataIndex: 'ip',
      key: 'ip',
    },
    {
      title: '端口',
      dataIndex: 'port',
      key: 'port',
    },
    {
      title: '标题',
      dataIndex: 'web_title',
      key: 'web_title',
    },
    {
      title: '国家',
      dataIndex: 'country',
      key: 'country',
    },
    {
      title: '省份',
      dataIndex: 'province',
      key: 'province',
    },
    {
      title: '城市',
      dataIndex: 'city',
      key: 'city',
    },
    {
      title: '服务器',
      dataIndex: 'server',
      key: 'server',
    },
  ];

  // 处理平台切换
  const handlePlatformChange = (value: string) => {
    setPlatform(value);
    setQuery('');
    setResults([]);
    setAutoCompleteOptions([]);
  };

  // 处理输入变化，生成联想提示
  const handleQueryChange = (value: string) => {
    setQuery(value);

    // 获取当前平台的语法提示
    const currentSyntax = syntaxHelp[platform as keyof typeof syntaxHelp];

    // 如果输入为空，显示所有提示
    if (!value.trim()) {
      const options = currentSyntax.map(item => ({
        value: item.label,
        label: (
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 500, color: 'var(--accent-cyan)' }}>{item.label}</span>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{item.description}</span>
          </div>
        ),
      }));
      setAutoCompleteOptions(options);
      return;
    }

    // 获取最后一个词（用于智能匹配）
    const lastWord = value.split(/[\s&|()]/).pop()?.toLowerCase() || '';

    // 过滤匹配的语法提示
    const filtered = currentSyntax.filter(item => {
      const label = item.label.toLowerCase();
      const desc = item.description.toLowerCase();
      return label.includes(lastWord) || desc.includes(lastWord);
    });

    // 生成联想选项
    const options = filtered.map(item => ({
      value: item.label,
      label: (
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontWeight: 500, color: 'var(--accent-cyan)' }}>{item.label}</span>
          <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{item.description}</span>
        </div>
      ),
    }));

    setAutoCompleteOptions(options);
  };

  // 处理选择联想项
  const handleSelect = (value: string) => {
    // 如果当前查询为空，直接设置
    if (!query.trim()) {
      setQuery(value);
      return;
    }

    // 否则追加到当前查询
    const connector = platform === 'quake' || platform === 'daydaymap' ? ' AND ' : ' && ';
    setQuery(query + connector + value);
  };

  // 处理搜索（带进度弹窗）
  const handleSearch = useCallback(async (searchPage?: number, searchPageSize?: number) => {
    if (!query) {
      return;
    }

    const page = searchPage ?? currentPage;
    const size = searchPageSize ?? pageSize;

    // 显示搜索进度弹窗
    setSearchModalOpen(true);
    setSearchStatus('running');
    setSearchPercent(30);
    setSearchStatusText(`正在查询 [${platform.toUpperCase()}] 第 ${page} 页...`);
    setSearchLogs([{
      time: new Date().toLocaleTimeString(),
      message: `发起查询: 平台=${platform}, 页码=${page}, 每页=${size}`,
      type: 'info',
    }]);

    setLoading(true);
    try {
      setSearchPercent(60);
      const result = await invoke('search_assets', {
        platform,
        query,
        page,
        pageSize: size,
      });

      const data = result as { total: number; results: AssetResult[] };

      setResults(data.results || []);
      setTotalResults(data.total || 0);

      setSearchPercent(100);
      setSearchStatus('success');
      setSearchStatusText(`查询完成！共找到 ${data.total || 0} 条结果，本页 ${(data.results || []).length} 条`);
      setSearchLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✓ 查询成功: 共 ${data.total || 0} 条结果`,
        type: 'success',
      }]);
    } catch (error: any) {
      console.error('查询出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`查询出错: ${errMsg}`);

      setSearchPercent(100);
      setSearchStatus('error');
      setSearchStatusText(`查询失败: ${errMsg}`);
      setSearchLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✗ 查询失败: ${errMsg}`,
        type: 'error',
      }]);
    } finally {
      setLoading(false);
    }
  }, [query, platform, currentPage, pageSize]);

  // 处理页码变化 - 切换页码后自动触发搜索
  const handlePageChange = (page: number, newPageSize?: number) => {
    setCurrentPage(page);
    if (newPageSize) setPageSize(newPageSize);
    // 直接用新的 page/pageSize 触发搜索
    handleSearch(page, newPageSize ?? pageSize);
  };

  // 应用地理位置筛选
  const applyLocationFilter = () => {
    let locationQuery = '';

    if (province) {
      switch (platform) {
        case 'hunter':
          locationQuery += `ip.province="${province}"`;
          break;
        case 'fofa':
          locationQuery += `region="${province}"`;
          break;
        case 'quake':
          locationQuery += `province: "${province}"`;
          break;
        case 'daydaymap':
          locationQuery += `province:"${province}"`;
          break;
      }
    }

    if (city) {
      if (locationQuery) {
        switch (platform) {
          case 'hunter':
          case 'fofa':
            locationQuery += ' && ';
            break;
          case 'quake':
          case 'daydaymap':
            locationQuery += ' AND ';
            break;
        }
      }

      switch (platform) {
        case 'hunter':
          locationQuery += `ip.city="${city}"`;
          break;
        case 'fofa':
          locationQuery += `city="${city}"`;
          break;
        case 'quake':
          locationQuery += `city: "${city}"`;
          break;
        case 'daydaymap':
          locationQuery += `city:"${city}"`;
          break;
      }
    }

    if (locationQuery) {
      if (appendToQuery && query) {
        switch (platform) {
          case 'hunter':
          case 'fofa':
            setQuery(`${query} && ${locationQuery}`);
            break;
          case 'quake':
          case 'daydaymap':
            setQuery(`${query} AND ${locationQuery}`);
            break;
        }
      } else {
        setQuery(locationQuery);
      }
    }
  };

  // 导出结果（带进度弹窗）
  const exportResults = async () => {
    if (results.length === 0) {
      message.warning('没有可导出的结果');
      return;
    }

    const taskId = `export_${Date.now()}`;

    // 重置并打开导出进度弹窗
    setExportModalOpen(true);
    setExportStatus('running');
    setExportPercent(0);
    setExportStatusText('正在准备导出...');
    setExportLogs([{
      time: new Date().toLocaleTimeString(),
      message: `开始导出: 平台=${platform}, 查询=${query}`,
      type: 'info',
    }]);
    setExportSummary([]);

    try {
      const filePath = await invoke<string>('export_results_with_progress', {
        taskId,
        platform,
        query,
        pages: 1,
        pageSize,
        timeRange: 'all',
      });
      message.success(`导出成功: ${filePath}`);
    } catch (error: any) {
      console.error('导出出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`导出出错: ${errMsg}`);
      // 如果后端没有发送 error 状态事件，手动设置
      setExportStatus(prev => prev === 'running' ? 'error' : prev);
      setExportStatusText(prev => prev.includes('失败') ? prev : `导出失败: ${errMsg}`);
      setExportLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✗ 导出失败: ${errMsg}`,
        type: 'error',
      }]);
    }
  };

  // 创建平台选项卡
  const tabItems = [
    { key: 'hunter', label: 'Hunter' },
    { key: 'fofa', label: 'FOFA' },
    { key: 'quake', label: 'Quake' },
    { key: 'daydaymap', label: 'DayDayMap' }
  ];

  // 创建页码选项
  const pageSizeOptions = [10, 20, 50, 100];

  return (

    <Card title="多平台资产查询" className="glass-effect" bordered={false}>
      <Tabs
        activeKey={platform}
        onChange={handlePlatformChange}
        items={tabItems}
        className="platform-tabs"
      />

      <div className="query-input-area" style={{ marginBottom: 24 }}>
        <AutoComplete
          style={{ width: '100%', marginBottom: 16 }}
          options={autoCompleteOptions}
          onSelect={handleSelect}
          onChange={handleQueryChange}
          value={query}
          placeholder={placeholders[platform as keyof typeof placeholders]}
          filterOption={false}
          popupClassName="glass-effect"
        >
          <Input.TextArea
            rows={3}
            placeholder={placeholders[platform as keyof typeof placeholders]}
            className="glass-effect"
            onPressEnter={(e) => {
              if (e.shiftKey) return; // Shift+Enter 换行
              e.preventDefault();
              handleSearch();
            }}
          />
        </AutoComplete>

        <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 16, flexWrap: 'wrap', gap: 16 }}>
          <Space wrap>
            <Select
              placeholder="选择省份"
              style={{ width: 150 }}
              value={province}
              onChange={setProvince}
              allowClear
              className="glass-effect"
            >
              <Select.Option value="北京市">北京市</Select.Option>
              <Select.Option value="上海市">上海市</Select.Option>
              <Select.Option value="广东省">广东省</Select.Option>
              <Select.Option value="浙江省">浙江省</Select.Option>
              <Select.Option value="江苏省">江苏省</Select.Option>
            </Select>

            <Select
              placeholder="选择城市"
              style={{ width: 150 }}
              value={city}
              onChange={setCity}
              allowClear
              className="glass-effect"
            >
              <Select.Option value="北京市">北京市</Select.Option>
              <Select.Option value="上海市">上海市</Select.Option>
              <Select.Option value="广州市">广州市</Select.Option>
              <Select.Option value="深圳市">深圳市</Select.Option>
              <Select.Option value="杭州市">杭州市</Select.Option>
            </Select>

            <Checkbox checked={appendToQuery} onChange={(e) => setAppendToQuery(e.target.checked)}>
              追加到当前查询
            </Checkbox>

            <Button type="default" onClick={applyLocationFilter} className="glass-effect">
              应用地理筛选
            </Button>
          </Space>

          <Space>
            <Select
              value={pageSize}
              onChange={(value) => setPageSize(value)}
              style={{ width: 120 }}
              className="glass-effect"
            >
              {pageSizeOptions.map(size => (
                <Select.Option key={size} value={size}>{size}条/页</Select.Option>
              ))}
            </Select>

            <Button
              type="primary"
              icon={<SearchOutlined />}
              onClick={() => handleSearch()}
              loading={loading}
              className="search-button gradient-button"
            >
              查询
            </Button>
          </Space>
        </div>
      </div>

      {totalResults > 0 && (
        <Alert
          message={
            <span>
              共找到 <span className="stats-number" style={{ fontSize: '18px' }}>{totalResults}</span> 个资产
            </span>
          }
          type="info"
          showIcon
          className="info-alert"
          style={{ marginBottom: 16 }}
        />
      )}

      <div className="results-table glass-effect" style={{ padding: 16, borderRadius: 8 }}>
        <Table
          columns={columns}
          dataSource={results}
          rowKey={(record) => record.ip + record.port}
          loading={loading}
          pagination={{
            current: currentPage,
            pageSize,
            total: totalResults,
            onChange: handlePageChange,
            showSizeChanger: true,
            pageSizeOptions: pageSizeOptions.map(size => size.toString()),
          }}
        />
      </div>

      {results.length > 0 && (
        <div style={{ marginTop: 16, textAlign: 'right' }}>
          <Button
            type="primary"
            icon={<DownloadOutlined />}
            onClick={exportResults}
            className="gradient-button"
          >
            导出结果
          </Button>
        </div>
      )}

      <Card title="语法提示" size="small" style={{ marginTop: 24 }} className="glass-effect" bordered={false}>
        <div className="syntax-help-tags">
          {syntaxHelp[platform as keyof typeof syntaxHelp].map((item, index) => (
            <div key={index} className="syntax-tag" onClick={() => handleSelect(item.label)}>
              <span style={{ color: 'var(--accent-cyan)', marginRight: 8, fontWeight: 'bold' }}>{item.label}</span>
              <span style={{ color: 'var(--text-secondary)' }}>{item.description}</span>
            </div>
          ))}
        </div>
      </Card>

      {/* 搜索进度弹窗 */}
      <ProgressModal
        open={searchModalOpen}
        title="资产查询"
        status={searchStatus}
        percent={searchPercent}
        statusText={searchStatusText}
        logs={searchLogs}
        onClose={() => setSearchModalOpen(false)}
      />

      {/* 导出进度弹窗 */}
      <ProgressModal
        open={exportModalOpen}
        title="数据导出"
        status={exportStatus}
        percent={exportPercent}
        statusText={exportStatusText}
        logs={exportLogs}
        summary={exportSummary}
        onClose={() => setExportModalOpen(false)}
      />
    </Card>
  );
};

export default AssetQuery; 