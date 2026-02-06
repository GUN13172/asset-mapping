import React from 'react';
import { Modal, Progress, Typography, Button, Space, Tag } from 'antd';
import {
  LoadingOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons';

const { Text, Paragraph } = Typography;

export type ProgressStatus = 'idle' | 'running' | 'success' | 'error' | 'cancelled';

export interface ProgressLog {
  time: string;
  message: string;
  type: 'info' | 'success' | 'error' | 'warning';
}

export interface ProgressModalProps {
  open: boolean;
  title: string;
  status: ProgressStatus;
  percent: number;
  statusText: string;
  logs: ProgressLog[];
  onCancel?: () => void;
  onClose: () => void;
  /** 额外的摘要信息，如总数、已完成数等 */
  summary?: { label: string; value: string | number }[];
}

const statusIcon: Record<ProgressStatus, React.ReactNode> = {
  idle: null,
  running: <LoadingOutlined spin style={{ color: 'var(--accent-cyan)', fontSize: 18 }} />,
  success: <CheckCircleOutlined style={{ color: 'var(--accent-green)', fontSize: 18 }} />,
  error: <CloseCircleOutlined style={{ color: 'var(--accent-red)', fontSize: 18 }} />,
  cancelled: <ExclamationCircleOutlined style={{ color: '#faad14', fontSize: 18 }} />,
};

const statusColor: Record<ProgressStatus, string> = {
  idle: 'default',
  running: 'processing',
  success: 'success',
  error: 'error',
  cancelled: 'warning',
};

const statusLabel: Record<ProgressStatus, string> = {
  idle: '准备中',
  running: '进行中',
  success: '已完成',
  error: '失败',
  cancelled: '已取消',
};

const logTypeColor: Record<string, string> = {
  info: 'var(--text-secondary)',
  success: 'var(--accent-green)',
  error: 'var(--accent-red)',
  warning: '#faad14',
};

const ProgressModal: React.FC<ProgressModalProps> = ({
  open,
  title,
  status,
  percent,
  statusText,
  logs,
  onCancel,
  onClose,
  summary,
}) => {
  const isFinished = status === 'success' || status === 'error' || status === 'cancelled';
  const progressStatus = status === 'error' ? 'exception' : status === 'success' ? 'success' : 'active';

  return (
    <Modal
      title={
        <Space>
          {statusIcon[status]}
          <span>{title}</span>
          <Tag color={statusColor[status]}>{statusLabel[status]}</Tag>
        </Space>
      }
      open={open}
      closable={isFinished}
      maskClosable={false}
      keyboard={false}
      onCancel={onClose}
      footer={
        <Space>
          {!isFinished && onCancel && (
            <Button danger onClick={onCancel}>
              取消
            </Button>
          )}
          {isFinished && (
            <Button type="primary" onClick={onClose}>
              关闭
            </Button>
          )}
        </Space>
      }
      width={600}
      className="progress-modal"
    >
      {/* 摘要信息 */}
      {summary && summary.length > 0 && (
        <div className="progress-summary">
          {summary.map((item, idx) => (
            <div key={idx} className="progress-summary-item">
              <Text type="secondary">{item.label}</Text>
              <Text strong>{item.value}</Text>
            </div>
          ))}
        </div>
      )}

      {/* 进度条 */}
      <div style={{ margin: '16px 0' }}>
        <Progress
          percent={Math.round(percent)}
          status={progressStatus}
          strokeColor={{
            '0%': '#00d4ff',
 '100%': '#7b2ff7',
          }}
          format={(p) => `${p}%`}
        />
      </div>

      {/* 状态文字 */}
      <Paragraph style={{ marginBottom: 12, color: 'var(--text-secondary)' }}>
        {statusText}
      </Paragraph>

      {/* 日志区域 */}
      {logs.length > 0 && (
        <div className="progress-log-container">
          {logs.map((log, idx) => (
            <div key={idx} className="progress-log-line">
              <Text style={{ color: 'var(--text-muted)', fontSize: 12, marginRight: 8, flexShrink: 0 }}>
                {log.time}
              </Text>
              <Text style={{ color: logTypeColor[log.type] || 'var(--text-secondary)', fontSize: 13 }}>
                {log.message}
              </Text>
            </div>
          ))}
        </div>
      )}
    </Modal>
  );
};

export default ProgressModal;

