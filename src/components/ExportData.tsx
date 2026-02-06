import React, { useState, useEffect, useRef } from 'react';
import { Card, Tabs, Input, Button, Select, DatePicker, Radio, Form, message, Space, Divider, Alert } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { RadioChangeEvent } from 'antd';
import dayjs from 'dayjs';
import ProgressModal, { ProgressStatus, ProgressLog } from './ProgressModal';

const { RangePicker } = DatePicker;
const { TextArea } = Input;

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

const ExportData: React.FC = () => {
  const [platform, setPlatform] = useState<string>('hunter');
  const [query, setQuery] = useState<string>('');
  const [pages, setPages] = useState<number>(10);
  const [pageSize, setPageSize] = useState<number>(100);
  const [timeRange, setTimeRange] = useState<string>('all');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);
  const [loading, setLoading] = useState<boolean>(false);
  const [exportType, setExportType] = useState<'current' | 'platform' | 'all'>('current');

  // 导出进度弹窗状态
  const [exportModalOpen, setExportModalOpen] = useState(false);
  const [exportStatus, setExportStatus] = useState<ProgressStatus>('idle');
  const [exportPercent, setExportPercent] = useState(0);
  const [exportStatusText, setExportStatusText] = useState('');
  const [exportLogs, setExportLogs] = useState<ProgressLog[]>([]);
  const [exportSummary, setExportSummary] = useState<{ label: string; value: string | number }[]>([]);

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
  const placeholders = {
    hunter: '例如: domain.suffix="test.com" && ip.province="北京市"',
    fofa: '例如: domain="test.com" && country="CN"',
    quake: '例如: domain: test.com AND country: "China"',
    daydaymap: '例如: domain:"test.com" AND region:"北京"',
  };

  // 处理平台切换
  const handlePlatformChange = (value: string) => {
    setPlatform(value);
    setQuery('');
  };

  // 处理时间范围切换
  const handleTimeRangeChange = (e: RadioChangeEvent) => {
    setTimeRange(e.target.value);
    if (e.target.value !== 'custom') {
      setDateRange([null, null]);
    }
  };

  // 处理日期范围变化
  const handleDateRangeChange = (dates: any) => {
    setDateRange(dates);
  };

  // 处理导出类型变化
  const handleExportTypeChange = (e: RadioChangeEvent) => {
    setExportType(e.target.value);
  };

  // 执行导出（带进度弹窗）
  const handleExport = async () => {
    if (!query) {
      message.warning('请输入查询语句');
      return;
    }

    const taskId = `export_${Date.now()}`;

    // 构建参数
    const params: any = {
      taskId,
      platform,
      query,
      pages,
      pageSize,
      timeRange,
    };

    // 如果是自定义时间范围，添加开始和结束日期
    if (timeRange === 'custom' && dateRange[0] && dateRange[1]) {
      params.startDate = dateRange[0].format('YYYY-MM-DD');
      params.endDate = dateRange[1].format('YYYY-MM-DD');
    }

    // 重置并打开进度弹窗
    setExportModalOpen(true);
    setExportStatus('running');
    setExportPercent(0);
    setExportStatusText('正在准备导出...');
    setExportLogs([{
      time: new Date().toLocaleTimeString(),
      message: `开始导出: 类型=${exportType}, 平台=${platform}, 页数=${pages}`,
      type: 'info',
    }]);
    setExportSummary([]);

    setLoading(true);
    try {
      // 对于 "current" 和 "platform" 类型，使用带进度的导出命令
      if (exportType === 'current' || exportType === 'platform') {
        const filePath = await invoke<string>('export_results_with_progress', params);
        message.success(`导出成功: ${filePath}`);
      } else {
        // "all" 类型暂时使用原有命令（不带进度事件）
        // 手动模拟进度
        setExportStatusText('正在导出所有平台数据...');
        await invoke('export_all_platforms', {
          query,
          pages,
          pageSize,
          timeRange,
          ...(params.startDate ? { startDate: params.startDate, endDate: params.endDate } : {}),
        });
        setExportPercent(100);
        setExportStatus('success');
        setExportStatusText('所有平台导出完成！');
        setExportLogs(prev => [...prev, {
          time: new Date().toLocaleTimeString(),
          message: '✓ 所有平台导出完成',
          type: 'success',
        }]);
        message.success('导出成功');
      }
    } catch (error: any) {
      console.error('导出出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`导出失败: ${errMsg}`);
      setExportStatus(prev => prev === 'running' ? 'error' : prev);
      setExportStatusText(prev => prev.includes('失败') ? prev : `导出失败: ${errMsg}`);
      setExportLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✗ 导出失败: ${errMsg}`,
        type: 'error',
      }]);
    } finally {
      setLoading(false);
    }
  };

  // 创建平台选项卡
  const tabItems = [
    { key: 'hunter', label: 'Hunter' },
    { key: 'fofa', label: 'FOFA' },
    { key: 'quake', label: 'Quake' },
    { key: 'daydaymap', label: 'DayDayMap' }
  ];

  // 创建页数选项
  const pageOptions = [
    { value: 5, label: '5页' },
    { value: 10, label: '10页' },
    { value: 20, label: '20页' },
    { value: 50, label: '50页' },
    { value: 100, label: '100页' }
  ];

  // 创建每页条数选项
  const pageSizeOptions = [
    { value: 10, label: '10条/页' },
    { value: 20, label: '20条/页' },
    { value: 50, label: '50条/页' },
    { value: 100, label: '100条/页' }
  ];

  return (
    <Card title="数据导出" variant="outlined">
      <Form layout="vertical">
        <Form.Item label="导出类型">
          <Radio.Group value={exportType} onChange={handleExportTypeChange}>
            <Radio.Button value="current">导出当前查询结果</Radio.Button>
            <Radio.Button value="platform">导出本平台全部资产</Radio.Button>
            <Radio.Button value="all">导出全部平台资产</Radio.Button>
          </Radio.Group>
        </Form.Item>

        {exportType !== 'all' && (
          <Form.Item label="选择平台">
            <Tabs activeKey={platform} onChange={handlePlatformChange} items={tabItems} />
          </Form.Item>
        )}

        <Form.Item label="查询语句">
          <TextArea
            placeholder={exportType !== 'all' ? placeholders[platform as keyof typeof placeholders] : '输入查询语句，将自动适配各平台语法'}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoSize={{ minRows: 2, maxRows: 6 }}
          />
        </Form.Item>

        <Form.Item label="导出设置">
          <Space direction="vertical" style={{ width: '100%' }}>
            <Space>
              <span>导出页数:</span>
              <Select
                value={pages}
                onChange={(value) => setPages(value)}
                style={{ width: 120 }}
                options={pageOptions}
              />
              
              <span>每页条数:</span>
              <Select
                value={pageSize}
                onChange={(value) => setPageSize(value)}
                style={{ width: 120 }}
                options={pageSizeOptions}
              />
            </Space>
          </Space>
        </Form.Item>

        <Form.Item label="时间范围">
          <Radio.Group value={timeRange} onChange={handleTimeRangeChange}>
            <Radio value="all">全部时间</Radio>
            <Radio value="1d">最近一天</Radio>
            <Radio value="7d">最近一周</Radio>
            <Radio value="30d">最近一个月</Radio>
            <Radio value="90d">最近三个月</Radio>
            <Radio value="365d">最近一年</Radio>
            <Radio value="custom">自定义时间</Radio>
          </Radio.Group>
          
          {timeRange === 'custom' && (
            <div style={{ marginTop: 16 }}>
              <RangePicker
                value={dateRange}
                onChange={handleDateRangeChange}
                style={{ width: '100%' }}
              />
            </div>
          )}
        </Form.Item>

        {exportType === 'all' && (
          <Alert
            message="注意"
            description="导出全部平台资产时，会自动根据不同平台调整查询语法。如果某些平台不支持特定语法，将自动跳过相关条件。"
            type="info"
            showIcon
            style={{ marginBottom: 16 }}
          />
        )}

        <Form.Item>
          <Button
            type="primary"
            icon={<DownloadOutlined />}
            onClick={handleExport}
            loading={loading}
            size="large"
          >
            开始导出
          </Button>
        </Form.Item>
      </Form>

      <Divider />

      <Card title="导出说明" size="small" variant="outlined">
        <ul>
          <li><strong>导出当前查询结果</strong>：仅导出当前查询条件下的资产数据</li>
          <li><strong>导出本平台全部资产</strong>：自动计算总页数，导出当前平台下符合条件的所有资产</li>
          <li><strong>导出全部平台资产</strong>：将查询语句适配到所有平台，并导出所有平台的资产</li>
          <li><strong>时间范围</strong>：限制导出资产的时间范围，不同平台的时间语法会自动适配</li>
        </ul>
      </Card>

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

export default ExportData; 