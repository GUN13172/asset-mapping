// 共享类型定义

export type ProgressEventStatus = 'running' | 'success' | 'error' | 'cancelled';
export type ProgressEventLogType = 'info' | 'success' | 'error' | 'warning';

// 导出进度事件结构体（对应后端 ProgressEvent）
export interface ProgressEventPayload {
    taskId: string;
    percent: number;
    status: ProgressEventStatus;
    statusText: string;
    logMessage?: string;
    logType?: ProgressEventLogType;
    currentPage?: number;
    totalPages?: number;
    totalResults?: number;
    fetchedResults?: number;
}
