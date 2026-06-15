import React, { useState } from 'react';
import { Card, Tabs, Input, Button, Select, DatePicker, Radio, Form, message, Space, Divider, Alert, InputNumber } from 'antd';
import { DownloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import type { RadioChangeEvent } from 'antd';
import dayjs from 'dayjs';
import ProgressModal from './ProgressModal';
import { endPerf, startPerf } from '../utils/perf';
import { normalizeSmartPunctuation } from '../utils/textInput';

const { RangePicker } = DatePicker;
const { TextArea } = Input;

import { useExportProgress } from '../hooks/useExportProgress';

const ExportData: React.FC = () => {
  const [platform, setPlatform] = useState<string>('hunter');
  const [query, setQuery] = useState<string>('');
  const [pages, setPages] = useState<number>(10);
  const [pageSize, setPageSize] = useState<number>(100);
  const [timeRange, setTimeRange] = useState<string>('all');
  const [dateRange, setDateRange] = useState<[dayjs.Dayjs | null, dayjs.Dayjs | null]>([null, null]);
  const [loading, setLoading] = useState<boolean>(false);
  const [exportType, setExportType] = useState<'current' | 'platform' | 'all'>('current');
  const [exportFormat, setExportFormat] = useState<'csv' | 'json'>('csv');

  // 导出进度 hook
  const exportProgress = useExportProgress();

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
    const normalizedQuery = normalizeSmartPunctuation(query).trim();
    if (!normalizedQuery) {
      message.warning('请输入查询语句');
      return;
    }
    if (normalizedQuery !== query) {
      setQuery(normalizedQuery);
    }

    const taskId = `export_${Date.now()}`;

    // 构建参数
    const params: any = {
      taskId,
      platform,
      query: normalizedQuery,
      pages,
      pageSize,
      timeRange,
      format: exportFormat,
    };

    // 如果是自定义时间范围，添加开始和结束日期
    if (timeRange === 'custom' && dateRange[0] && dateRange[1]) {
      params.startDate = dateRange[0].format('YYYY-MM-DD');
      params.endDate = dateRange[1].format('YYYY-MM-DD');
    }

    // 重置并打开进度弹窗
    exportProgress.startTask(taskId, `开始导出: 类型=${exportType}, 平台=${platform}, 页数=${pages}`);
    const exportPerfToken = startPerf('asset-export', {
      source: 'export-center',
      exportType,
      platform,
      pages,
      pageSize,
      format: exportFormat,
    });

    setLoading(true);
    try {
      // 对于 "current" 和 "platform" 类型，使用带进度的导出命令
      if (exportType === 'current' || exportType === 'platform') {
        const filePath = await invoke<string>('export_results_with_progress', params);
        message.success(`导出成功: ${filePath}`);
        endPerf(exportPerfToken, {
          source: 'export-center',
          exportType,
          platform,
          status: 'success',
        });
      } else {
        // "all" 类型：按顺序调用各平台导出，确保进度弹窗一致
        exportProgress.setStatusText('正在准备导出所有平台...');
        const platforms = ['hunter', 'fofa', 'quake', 'daydaymap'];
        // 因为没有指定源平台，我们假设当前选中的标签页就是源平台
        const sourcePlatform = platform;

        for (let i = 0; i < platforms.length; i++) {
          const targetPlatform = platforms[i];
          let convertedQuery = normalizedQuery;

          // 尝试将语法适应目标平台
          if (sourcePlatform !== targetPlatform) {
            try {
              const convertRes = await invoke<{ platform: string, query: string }[]>('convert_query_to_all', {
                query: normalizedQuery,
                fromPlatform: sourcePlatform,
              });
              const target = convertRes.find(r => r.platform === targetPlatform);
              if (target) {
                convertedQuery = target.query;
              }
            } catch (e) {
              exportProgress.addLog(`⚠ ${targetPlatform} 查询语法转换提示: ${e}`, 'warning');
              // 继续往下走，也许语法本就兼容
            }
          }

          exportProgress.setStatusText(`正在导出 ${targetPlatform} 平台数据 (${i + 1}/${platforms.length})...`);
          try {
            const filePath = await invoke<string>('export_results_with_progress', {
              ...params,
              platform: targetPlatform,
              query: convertedQuery,
              taskId: `${taskId}_${targetPlatform}`,
            });
            exportProgress.addLog(`✓ ${targetPlatform} 导出成功: ${filePath}`, 'success');
          } catch (err: any) {
            exportProgress.addLog(`✗ ${targetPlatform} 导出失败: ${err}`, 'error');
          }
        }

        exportProgress.setPercent(100);
        exportProgress.setStatus('success');
        exportProgress.setStatusText('所有平台导出任务结束！');
        exportProgress.addLog('✓ 全部平台导出完毕', 'success');
        exportProgress.finishTask();
        message.success('导出成功，结果请查看日志路径');
        endPerf(exportPerfToken, {
          source: 'export-center',
          exportType,
          platform,
          status: 'success',
        });
      }
    } catch (error: any) {
      console.error('导出出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`导出失败: ${errMsg}`);
      exportProgress.setStatus('error');
      exportProgress.setStatusText(`导出失败: ${errMsg}`);
      exportProgress.addLog(`✗ 导出失败: ${errMsg}`, 'error');
      exportProgress.finishTask();
      endPerf(exportPerfToken, {
        source: 'export-center',
        exportType,
        platform,
        status: 'error',
        error: errMsg,
      });
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

  // 创建每页条数选项
  const pageSizeOptions = [
    { value: 10, label: '10条/页' },
    { value: 20, label: '20条/页' },
    { value: 50, label: '50条/页' },
    { value: 100, label: '100条/页' }
  ];

  return (
    <Card title="数据导出" className="glass-effect" bordered={false}>
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
            onChange={(e) => setQuery(normalizeSmartPunctuation(e.target.value))}
            autoSize={{ minRows: 2, maxRows: 6 }}
            autoCorrect="off"
            autoCapitalize="off"
            autoComplete="off"
            spellCheck={false}
          />
        </Form.Item>

        <Form.Item label="导出设置">
          <Space direction="vertical" style={{ width: '100%' }}>
            <Space>
              <span>导出页数:</span>
              <InputNumber
                value={pages}
                min={1}
                max={100}
                onChange={(value) => setPages(value ?? 1)}
                style={{ width: 120 }}
              />

              <span>每页条数:</span>
              <Select
                value={pageSize}
                onChange={(value) => setPageSize(value)}
                style={{ width: 120 }}
                options={pageSizeOptions}
              />
            </Space>

            <span style={{ color: 'var(--text-secondary)', fontSize: 12 }}>
              支持自定义 1-100 页
            </span>

            <Space>
              <span>导出格式:</span>
              <Radio.Group
                value={exportFormat}
                onChange={(e) => setExportFormat(e.target.value)}
                optionType="button"
                buttonStyle="solid"
                size="small"
              >
                <Radio.Button value="csv">CSV</Radio.Button>
                <Radio.Button value="json">JSON</Radio.Button>
              </Radio.Group>
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

      <Card title="导出说明" size="small" className="glass-effect" bordered={false}>
        <ul>
          <li><strong>导出当前查询结果</strong>：仅导出当前查询条件下的资产数据</li>
          <li><strong>导出本平台全部资产</strong>：自动计算总页数，导出当前平台下符合条件的所有资产</li>
          <li><strong>导出全部平台资产</strong>：将查询语句适配到所有平台，并导出所有平台的资产</li>
          <li><strong>时间范围</strong>：限制导出资产的时间范围，不同平台的时间语法会自动适配</li>
        </ul>
      </Card>

      {/* 导出进度弹窗 */}
      <ProgressModal
        open={exportProgress.modalOpen}
        title="数据导出"
        status={exportProgress.status}
        percent={exportProgress.percent}
        statusText={exportProgress.statusText}
        logs={exportProgress.logs}
        summary={exportProgress.summary}
        onClose={() => exportProgress.setModalOpen(false)}
      />
    </Card>
  );
};

export default ExportData;
