import React, { useState, useEffect } from 'react';
import {
  Alert,
  Card,
  InputNumber,
  Table,
  Button,
  Input,
  Select,
  Space,
  Tag,
  Popconfirm,
  message,
  Tooltip,
  Modal,
  Typography
} from 'antd';
import {
  HistoryOutlined,
  DeleteOutlined,
  ExportOutlined,
  SearchOutlined,
  ReloadOutlined,
  ClearOutlined,
  EyeOutlined,
  CopyOutlined
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import dayjs from 'dayjs';
import ProgressModal from './ProgressModal';
import { useExportProgress } from '../hooks/useExportProgress';

const { Title, Text } = Typography;
const EXPORT_PAGE_PRESETS = [5, 10, 20, 50, 100];

interface QueryHistory {
  id: string;
  platform: string;
  query: string;
  results_count: number;
  timestamp: string;
  success: boolean;
  error_message?: string;
}

interface HistoryRecordsProps {
  active?: boolean;
}

const HistoryRecords: React.FC<HistoryRecordsProps> = ({ active = true }) => {
  const [history, setHistory] = useState<QueryHistory[]>([]);
  const [filteredHistory, setFilteredHistory] = useState<QueryHistory[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [selectedPlatform, setSelectedPlatform] = useState<string>('all');
  const [searchKeyword, setSearchKeyword] = useState<string>('');
  const [detailModalVisible, setDetailModalVisible] = useState<boolean>(false);
  const [selectedRecord, setSelectedRecord] = useState<QueryHistory | null>(null);
  const [exportConfigVisible, setExportConfigVisible] = useState<boolean>(false);
  const [exportTargetRecord, setExportTargetRecord] = useState<QueryHistory | null>(null);
  const [exportPageSize, setExportPageSize] = useState<number>(100);
  const [exportPages, setExportPages] = useState<number>(50);
  const [exportExactPages, setExportExactPages] = useState<number>(1);
  const [exportRecommendedPages, setExportRecommendedPages] = useState<number>(50);
  const [exportPageMode, setExportPageMode] = useState<'recommended' | 'exact' | 'manual'>('recommended');
  const [exportFormat, setExportFormat] = useState<'csv' | 'json'>('csv');
  const [exportSubmitting, setExportSubmitting] = useState<boolean>(false);
  const exportProgress = useExportProgress();

  useEffect(() => {
    if (!active) {
      return;
    }

    loadHistory();
  }, [active]);

  useEffect(() => {
    filterHistory();
  }, [history, selectedPlatform, searchKeyword]);

  // 加载历史记录
  const loadHistory = async () => {
    setLoading(true);
    try {
      const records = await invoke<QueryHistory[]>('get_query_history');
      setHistory(records);
    } catch (error) {
      console.error('加载历史记录失败:', error);
      message.error(`加载历史记录失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 筛选历史记录
  const filterHistory = () => {
    let filtered = [...history];

    // 按平台筛选
    if (selectedPlatform !== 'all') {
      filtered = filtered.filter(item => item.platform === selectedPlatform);
    }

    // 按关键词搜索
    if (searchKeyword) {
      const keyword = searchKeyword.toLowerCase();
      filtered = filtered.filter(
        item =>
          item.query.toLowerCase().includes(keyword) ||
          item.platform.toLowerCase().includes(keyword)
      );
    }

    setFilteredHistory(filtered);
  };

  // 删除单条记录
  const deleteRecord = async (id: string) => {
    try {
      await invoke('delete_query_history', { id });
      message.success('删除成功');
      loadHistory();
    } catch (error) {
      console.error('删除失败:', error);
      message.error(`删除失败: ${error}`);
    }
  };

  // 清空所有记录
  const clearAllRecords = async () => {
    try {
      await invoke('clear_all_history');
      message.success('清空成功');
      loadHistory();
    } catch (error) {
      console.error('清空失败:', error);
      message.error(`清空失败: ${error}`);
    }
  };

  // 导出历史记录
  const exportHistory = async () => {
    try {
      const exportPath = await invoke<string>('select_directory');
      if (!exportPath) return;

      const filePath = await invoke<string>('export_query_history', {
        exportPath
      });

      message.success(`导出成功：${filePath}`);
    } catch (error) {
      console.error('导出失败:', error);
      message.error(`导出失败: ${error}`);
    }
  };

  const calculateExportPlan = (totalResults: number, pageSize: number) => {
    const exactPages = Math.max(1, Math.ceil(totalResults / pageSize));
    const recommendedPages =
      EXPORT_PAGE_PRESETS.find((preset) => preset >= exactPages) ??
      EXPORT_PAGE_PRESETS[EXPORT_PAGE_PRESETS.length - 1];

    return {
      exactPages,
      recommendedPages,
    };
  };

  const openExportConfig = (record: QueryHistory) => {
    const defaultPageSize = 100;
    const { exactPages, recommendedPages } = calculateExportPlan(record.results_count, defaultPageSize);
    setExportTargetRecord(record);
    setExportPageSize(defaultPageSize);
    setExportExactPages(exactPages);
    setExportRecommendedPages(recommendedPages);
    setExportPageMode('recommended');
    setExportFormat('csv');
    setExportPages(recommendedPages);
    setExportConfigVisible(true);
  };

  const handleExportPageSizeChange = (value: number | null) => {
    if (!exportTargetRecord) {
      return;
    }

    const nextPageSize = value ?? 100;
    const { exactPages, recommendedPages } = calculateExportPlan(
      exportTargetRecord.results_count,
      nextPageSize
    );
    setExportPageSize(nextPageSize);
    setExportExactPages(exactPages);
    setExportRecommendedPages(recommendedPages);
    setExportPages((currentPages) => {
      if (exportPageMode === 'exact') {
        return Math.min(100, exactPages);
      }
      if (exportPageMode === 'recommended') {
        return recommendedPages;
      }
      return Math.max(1, Math.min(100, currentPages));
    });
  };

  const handleExportPagesChange = (value: number | null) => {
    setExportPageMode('manual');
    setExportPages(Math.max(1, Math.min(100, value ?? 1)));
  };

  // 导出单条记录的资产
  const exportRecordAssets = async (record: QueryHistory, pages: number, pageSize: number) => {
    try {
      // 生成唯一的任务ID
      const taskId = `export_${record.platform}_${Date.now()}`;
      const pagesToExport = Math.max(1, Math.min(100, pages));

      exportProgress.startTask(
        taskId,
        `开始导出历史记录: 平台=${record.platform}, 页数=${pagesToExport}, 格式=${exportFormat.toUpperCase()}`
      );

      // 调用导出接口
      const filePath = await invoke<string>('export_results_with_progress', {
        taskId: taskId,
        platform: record.platform,
        query: record.query,
        pages: pagesToExport,
        pageSize: pageSize,
        timeRange: 'all',
        startDate: null,
        endDate: null,
        format: exportFormat,
      });
      message.success(`导出成功: ${filePath}`);
    } catch (error) {
      console.error('导出资产失败:', error);
      const errMsg = typeof error === 'string' ? error : '未知错误';
      message.error({ content: `导出失败: ${errMsg}`, duration: 3 });
      exportProgress.setStatus('error');
      exportProgress.setStatusText(`导出失败: ${errMsg}`);
      exportProgress.addLog(`导出失败: ${errMsg}`, 'error');
      exportProgress.finishTask();
    }
  };

  const confirmExportRecordAssets = async () => {
    if (!exportTargetRecord) {
      return;
    }

    setExportSubmitting(true);
    setExportConfigVisible(false);
    try {
      await exportRecordAssets(exportTargetRecord, exportPages, exportPageSize);
    } finally {
      setExportSubmitting(false);
    }
  };

  // 复制查询语句
  const copyQuery = async (query: string) => {
    try {
      // 尝试使用浏览器API
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(query);
        message.success('已复制到剪贴板');
      } else {
        // 降级方案：使用传统方法
        const textArea = document.createElement('textarea');
        textArea.value = query;
        textArea.style.position = 'fixed';
        textArea.style.left = '-999999px';
        textArea.style.top = '-999999px';
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        try {
          document.execCommand('copy');
          message.success('已复制到剪贴板');
        } catch (err) {
          message.error('复制失败，请手动复制');
        }
        document.body.removeChild(textArea);
      }
    } catch (error) {
      console.error('复制失败:', error);
      message.error('复制失败，请手动复制');
    }
  };

  // 查看详情
  const viewDetails = (record: QueryHistory) => {
    setSelectedRecord(record);
    setDetailModalVisible(true);
  };

  // 平台标签颜色映射
  const platformColors: Record<string, string> = {
    hunter: 'orange',
    fofa: 'blue',
    quake: 'purple',
    daydaymap: 'cyan'
  };

  // 表格列定义
  const columns = [
    {
      title: '时间',
      dataIndex: 'timestamp',
      key: 'timestamp',
      width: 180,
      render: (text: string) => {
        const date = new Date(text);
        return (
          <Tooltip title={date.toLocaleString('zh-CN')}>
            <Text>{dayjs(date).format('YYYY-MM-DD HH:mm:ss')}</Text>
          </Tooltip>
        );
      },
    },
    {
      title: '平台',
      dataIndex: 'platform',
      key: 'platform',
      width: 100,
      render: (platform: string) => (
        <Tag color={platformColors[platform.toLowerCase()] || 'default'}>
          {platform.toUpperCase()}
        </Tag>
      ),
    },
    {
      title: '查询语句',
      dataIndex: 'query',
      key: 'query',
      ellipsis: {
        showTitle: false,
      },
      render: (query: string) => (
        <Tooltip title={query} placement="topLeft">
          <Text code style={{ maxWidth: 400 }}>{query}</Text>
        </Tooltip>
      ),
    },
    {
      title: '结果数',
      dataIndex: 'results_count',
      key: 'results_count',
      width: 100,
      render: (count: number) => (
        <Text strong>{count.toLocaleString()}</Text>
      ),
    },
    {
      title: '状态',
      dataIndex: 'success',
      key: 'success',
      width: 80,
      render: (success: boolean, record: QueryHistory) => (
        <Tooltip title={record.error_message || ''}>
          <Tag color={success ? 'success' : 'error'}>
            {success ? '成功' : '失败'}
          </Tag>
        </Tooltip>
      ),
    },
    {
      title: '操作',
      key: 'action',
      width: 220,
      render: (_: any, record: QueryHistory) => (
        <Space size="small">
          <Tooltip title="查看详情">
            <Button
              type="link"
              size="small"
              icon={<EyeOutlined />}
              onClick={() => viewDetails(record)}
            />
          </Tooltip>
          <Tooltip title="复制查询">
            <Button
              type="link"
              size="small"
              icon={<CopyOutlined />}
              onClick={() => copyQuery(record.query)}
            />
          </Tooltip>
          <Tooltip title="导出资产">
            <Button
              type="link"
              size="small"
              icon={<ExportOutlined />}
              onClick={() => openExportConfig(record)}
              disabled={!record.success || record.results_count === 0}
            />
          </Tooltip>
          <Popconfirm
            title="确定要删除这条记录吗？"
            onConfirm={() => deleteRecord(record.id)}
            okText="确定"
            cancelText="取消"
          >
            <Tooltip title="删除">
              <Button
                type="link"
                size="small"
                danger
                icon={<DeleteOutlined />}
              />
            </Tooltip>
          </Popconfirm>
        </Space>
      ),
    },
  ];

  const estimatedExportCount = exportTargetRecord
    ? Math.min(exportTargetRecord.results_count, exportPages * exportPageSize)
    : 0;
  const remainingExportCount = exportTargetRecord
    ? Math.max(0, exportTargetRecord.results_count - estimatedExportCount)
    : 0;
  const isFullCoverage = exportTargetRecord
    ? estimatedExportCount >= exportTargetRecord.results_count
    : false;

  return (
    <div className="history-records">
      <Card
        className="glass-effect"
        bordered={false}
        title={
          <Space>
            <HistoryOutlined />
            <Title level={4} style={{ margin: 0 }}>历史查询记录</Title>
          </Space>
        }
        extra={
          <Space>
            <Button
              icon={<ReloadOutlined />}
              onClick={loadHistory}
              loading={loading}
            >
              刷新
            </Button>
            <Button
              icon={<ExportOutlined />}
              onClick={exportHistory}
              disabled={history.length === 0}
            >
              导出
            </Button>
            <Popconfirm
              title="确定要清空所有历史记录吗？此操作不可恢复！"
              onConfirm={clearAllRecords}
              okText="确定"
              cancelText="取消"
            >
              <Button
                icon={<ClearOutlined />}
                danger
                disabled={history.length === 0}
              >
                清空
              </Button>
            </Popconfirm>
          </Space>
        }
      >
        {/* 筛选栏 */}
        <Space style={{ marginBottom: 16, width: '100%', justifyContent: 'space-between' }}>
          <Space>
            <Select
              style={{ width: 150 }}
              value={selectedPlatform}
              onChange={setSelectedPlatform}
              options={[
                { label: '全部平台', value: 'all' },
                { label: 'Hunter', value: 'hunter' },
                { label: 'FOFA', value: 'fofa' },
                { label: 'Quake', value: 'quake' },
                { label: 'DayDayMap', value: 'daydaymap' },
              ]}
            />
            <Input
              placeholder="搜索查询语句或平台..."
              prefix={<SearchOutlined />}
              allowClear
              style={{ width: 300 }}
              value={searchKeyword}
              onChange={(e) => setSearchKeyword(e.target.value)}
            />
          </Space>
          <Text type="secondary">
            共 {filteredHistory.length} 条记录
          </Text>
        </Space>

        {/* 表格 */}
        <Table
          columns={columns}
          dataSource={filteredHistory}
          rowKey="id"
          loading={loading}
          pagination={{
            pageSize: 10,
            showSizeChanger: true,
            pageSizeOptions: ['10', '20', '50', '100'],
            showQuickJumper: true,
            showTotal: (total) => `共 ${total} 条记录`,
          }}
        />
      </Card>

      {/* 详情模态框 */}
      <Modal
        title="查询详情"
        open={detailModalVisible}
        onCancel={() => setDetailModalVisible(false)}
        footer={[
          <Button key="copy" icon={<CopyOutlined />} onClick={() => selectedRecord && copyQuery(selectedRecord.query)}>
            复制查询
          </Button>,
          <Button key="close" type="primary" onClick={() => setDetailModalVisible(false)}>
            关闭
          </Button>,
        ]}
        width={700}
      >
        {selectedRecord && (
          <div>
            <p><Text strong>平台：</Text> <Tag color={platformColors[selectedRecord.platform.toLowerCase()]}>{selectedRecord.platform.toUpperCase()}</Tag></p>
            <p><Text strong>时间：</Text> {dayjs(selectedRecord.timestamp).format('YYYY-MM-DD HH:mm:ss')}</p>
            <p><Text strong>查询语句：</Text></p>
            <pre style={{ background: '#f5f5f5', padding: 12, borderRadius: 4 }}>{selectedRecord.query}</pre>
            <p><Text strong>结果数量：</Text> {selectedRecord.results_count.toLocaleString()} 条</p>
            <p><Text strong>状态：</Text> <Tag color={selectedRecord.success ? 'success' : 'error'}>{selectedRecord.success ? '成功' : '失败'}</Tag></p>
            {selectedRecord.error_message && (
              <div>
                <Text strong>错误信息：</Text>
                <pre style={{ background: '#fff2f0', padding: 12, borderRadius: 4, color: '#cf1322' }}>
                  {selectedRecord.error_message}
                </pre>
              </div>
            )}
          </div>
        )}
      </Modal>

      <Modal
        title="导出资产设置"
        open={exportConfigVisible}
        onCancel={() => !exportSubmitting && setExportConfigVisible(false)}
        onOk={confirmExportRecordAssets}
        confirmLoading={exportSubmitting}
        okText="开始导出"
        cancelText="取消"
      >
        {exportTargetRecord && (
          <Space direction="vertical" size={16} style={{ width: '100%' }}>
            <Alert
              type="info"
              showIcon
              message={`共 ${exportTargetRecord.results_count.toLocaleString()} 条资产`}
              description={
                exportExactPages > 100
                  ? `按 ${exportPageSize} 条/页完整导出需要 ${exportExactPages} 页，当前建议导出前 ${exportRecommendedPages} 页；你也可以手动调整，最大支持 100 页。`
                  : `按 ${exportPageSize} 条/页精确导出需要 ${exportExactPages} 页。已为你智能推荐 ${exportRecommendedPages} 页，你也可以手动改成 ${exportExactPages} 页。`
              }
            />

            <div>
              <Text strong>平台：</Text>
              <Tag color={platformColors[exportTargetRecord.platform.toLowerCase()]}>
                {exportTargetRecord.platform.toUpperCase()}
              </Tag>
            </div>

            <div>
              <Text strong>每页条数</Text>
              <div style={{ marginTop: 8 }}>
                <Select
                  value={exportPageSize}
                  onChange={handleExportPageSizeChange}
                  style={{ width: 160 }}
                  options={[
                    { label: '10 条/页', value: 10 },
                    { label: '20 条/页', value: 20 },
                    { label: '50 条/页', value: 50 },
                    { label: '100 条/页', value: 100 },
                  ]}
                />
              </div>
            </div>

            <div>
              <Text strong>导出格式</Text>
              <div style={{ marginTop: 8 }}>
                <Select
                  value={exportFormat}
                  onChange={setExportFormat}
                  style={{ width: 160 }}
                  options={[
                    { label: 'CSV', value: 'csv' },
                    { label: 'JSON', value: 'json' },
                  ]}
                />
              </div>
            </div>

            <div>
              <Text strong>导出页数</Text>
              <Space direction="vertical" size={8} style={{ width: '100%', marginTop: 8 }}>
                <InputNumber
                  min={1}
                  max={100}
                  value={exportPages}
                  onChange={handleExportPagesChange}
                  style={{ width: 160 }}
                />
                <Space wrap>
                  <Button
                    size="small"
                    onClick={() => {
                      setExportPageMode('exact');
                      setExportPages(Math.min(100, exportExactPages));
                    }}
                  >
                    精确页数 {Math.min(100, exportExactPages)}
                  </Button>
                  <Button
                    size="small"
                    type="primary"
                    ghost
                    onClick={() => {
                      setExportPageMode('recommended');
                      setExportPages(exportRecommendedPages);
                    }}
                  >
                    推荐页数 {exportRecommendedPages}
                  </Button>
                </Space>
                <Space direction="vertical" size={2}>
                  <Text type="secondary">
                    精确需要页数：{exportExactPages.toLocaleString()}
                    {exportExactPages > 100 ? '（超出当前 100 页上限）' : ''}
                  </Text>
                  <Text type="secondary">智能推荐页数：{exportRecommendedPages}</Text>
                  <Text type="secondary">当前导出页数：{exportPages}</Text>
                </Space>
              </Space>
            </div>

            <Alert
              type={isFullCoverage ? 'success' : 'warning'}
              showIcon
              message={
                isFullCoverage
                  ? `当前设置预计导出 ${estimatedExportCount.toLocaleString()} / ${exportTargetRecord.results_count.toLocaleString()} 条资产（完整覆盖）`
                  : `当前设置预计导出 ${estimatedExportCount.toLocaleString()} / ${exportTargetRecord.results_count.toLocaleString()} 条资产`
              }
              description={
                isFullCoverage
                  ? `按 ${exportPageSize} 条/页导出 ${exportPages} 页即可覆盖全部结果。`
                  : exportExactPages > 100
                    ? `按当前每页条数完整导出需要 ${exportExactPages} 页，已超出 100 页上限；本次预计还差 ${remainingExportCount.toLocaleString()} 条资产。`
                    : `当前还差 ${remainingExportCount.toLocaleString()} 条资产；如需完整导出，可将页数调整为 ${exportExactPages} 页。`
              }
            />
          </Space>
        )}
      </Modal>

      {/* 导出进度弹窗 */}
      <ProgressModal
        open={exportProgress.modalOpen}
        onClose={() => exportProgress.setModalOpen(false)}
        title="导出资产"
        status={exportProgress.status}
        percent={exportProgress.percent}
        statusText={exportProgress.statusText}
        logs={exportProgress.logs}
        summary={exportProgress.summary}
      />
    </div>
  );
};

export default HistoryRecords;
