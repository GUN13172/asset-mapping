import React, { useState, useEffect } from 'react';
import {
  Card,
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
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import dayjs from 'dayjs';
import ProgressModal from './ProgressModal';

const { Title, Text } = Typography;

interface QueryHistory {
  id: string;
  platform: string;
  query: string;
  results_count: number;
  timestamp: string;
  success: boolean;
  error_message?: string;
}

interface ProgressStatus {
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

const HistoryRecords: React.FC = () => {
  const [history, setHistory] = useState<QueryHistory[]>([]);
  const [filteredHistory, setFilteredHistory] = useState<QueryHistory[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [selectedPlatform, setSelectedPlatform] = useState<string>('all');
  const [searchKeyword, setSearchKeyword] = useState<string>('');
  const [detailModalVisible, setDetailModalVisible] = useState<boolean>(false);
  const [selectedRecord, setSelectedRecord] = useState<QueryHistory | null>(null);
  
  // 导出进度状态
  const [exportProgressVisible, setExportProgressVisible] = useState<boolean>(false);
  const [exportProgress, setExportProgress] = useState<ProgressStatus>({
    taskId: '',
    percent: 0,
    status: 'idle',
    statusText: '准备中...',
  });
  const [exportLogs, setExportLogs] = useState<Array<{ type: 'success' | 'error' | 'info' | 'warning'; message: string; time: string }>>([]);

  useEffect(() => {
    loadHistory();
    
    // 监听导出进度事件
    const unlisten = listen<ProgressStatus>('export-progress', (event) => {
      const progress = event.payload;
      setExportProgress(progress);
      
      // 添加日志
      if (progress.logMessage) {
        const logType = (progress.logType || 'info') as 'success' | 'error' | 'info' | 'warning';
        setExportLogs(prev => [...prev, {
          type: logType,
          message: progress.logMessage!,
          time: new Date().toLocaleTimeString()
        }]);
      }
      
      // 如果完成或失败，3秒后关闭弹窗
      if (progress.status === 'completed' || progress.status === 'failed') {
        setTimeout(() => {
          setExportProgressVisible(false);
          setExportLogs([]);
        }, 3000);
      }
    });
    
    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

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

  // 导出单条记录的资产
  const exportRecordAssets = async (record: QueryHistory) => {
    try {
      // 生成唯一的任务ID
      const taskId = `export_${record.platform}_${Date.now()}`;
      
      // 根据结果数量自动计算需要导出的页数（全部导出）
      const pageSize = 100;
      const totalResults = record.results_count;
      const pagesToExport = Math.ceil(totalResults / pageSize);
      
      console.log(`📊 导出计算: 总结果=${totalResults}, 每页=${pageSize}, 导出页数=${pagesToExport}页`);
      
      // 重置进度状态
      setExportProgress({
        taskId: taskId,
        percent: 0,
        status: 'running',
        statusText: '准备导出...',
      });
      setExportLogs([]);
      setExportProgressVisible(true);

      // 调用导出接口
      await invoke('export_results_with_progress', {
        taskId: taskId,
        platform: record.platform,
        query: record.query,
        pages: pagesToExport,
        pageSize: pageSize,
        timeRange: 'all',
        startDate: null,
        endDate: null
      });
    } catch (error) {
      console.error('导出资产失败:', error);
      message.error({ content: `导出失败: ${error}`, duration: 3 });
      setExportProgressVisible(false);
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
              onClick={() => exportRecordAssets(record)}
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

  return (
    <div className="history-records">
      <Card
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

      {/* 导出进度弹窗 */}
      <ProgressModal
        open={exportProgressVisible}
        onClose={() => setExportProgressVisible(false)}
        title="导出资产"
        status={exportProgress.status as 'idle' | 'running' | 'success' | 'error' | 'cancelled'}
        percent={exportProgress.percent}
        statusText={exportProgress.statusText}
        logs={exportLogs}
        summary={exportProgress.totalPages ? [
          { label: '当前页', value: `${exportProgress.currentPage || 0}/${exportProgress.totalPages}` },
          { label: '已获取', value: `${exportProgress.fetchedResults || 0}` },
          { label: '总结果', value: `${exportProgress.totalResults || 0}` },
        ] : undefined}
      />
    </div>
  );
};

export default HistoryRecords;

