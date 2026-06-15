import { useState, useEffect, useCallback, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { ProgressStatus, ProgressLog } from '../components/ProgressModal';
import type { ProgressEventPayload } from '../types';

// 摘要项类型
export interface SummaryItem {
    label: string;
    value: string | number;
}

/**
 * 导出进度监听 Hook
 * 封装 `export-progress` 事件的监听和状态管理逻辑，
 * 消除各导出入口中的重复代码。
 */
export function useExportProgress() {
    const [modalOpen, setModalOpen] = useState(false);
    const [status, setStatus] = useState<ProgressStatus>('idle');
    const [percent, setPercent] = useState(0);
    const [statusText, setStatusText] = useState('');
    const [logs, setLogs] = useState<ProgressLog[]>([]);
    const [summary, setSummary] = useState<SummaryItem[]>([]);
    const activeTaskIdRef = useRef<string | null>(null);

    // 组件挂载后保持一个轻量监听器，避免任务刚启动时错过后端立即发出的首个进度事件。
    useEffect(() => {
        let disposed = false;
        let unlistenFn: (() => void) | null = null;

        listen<ProgressEventPayload>('export-progress', (event) => {
                const activeTaskId = activeTaskIdRef.current;
                if (!activeTaskId) {
                    return;
                }

                const data = event.payload;
                if (
                    (data.taskId !== activeTaskId && !data.taskId.startsWith(`${activeTaskId}_`))
                ) {
                    return;
                }
                setPercent(data.percent);
                setStatus(data.status);
                setStatusText(data.statusText);

                if (data.logMessage) {
                    const now = new Date().toLocaleTimeString();
                    setLogs(prev => [...prev, {
                        time: now,
                        message: data.logMessage!,
                        type: data.logType || 'info',
                    }]);
                }

                // 更新摘要
                const summaryItems: SummaryItem[] = [];
                if (data.totalPages != null) summaryItems.push({ label: '总页数', value: data.totalPages });
                if (data.currentPage != null) summaryItems.push({ label: '当前页', value: data.currentPage });
                if (data.totalResults != null) summaryItems.push({ label: '总结果数', value: data.totalResults });
                if (data.fetchedResults != null) summaryItems.push({ label: '已获取', value: data.fetchedResults });
                if (summaryItems.length > 0) setSummary(summaryItems);

                if (
                    data.taskId === activeTaskId &&
                    (data.status === 'success' || data.status === 'error' || data.status === 'cancelled')
                ) {
                    activeTaskIdRef.current = null;
                }
            }).then((unlisten) => {
            if (disposed) {
                unlisten();
                return;
            }
            unlistenFn = unlisten;
        }).catch((error) => {
            console.error('监听导出进度失败:', error);
        });

        return () => {
            disposed = true;
            unlistenFn?.();
        };
    }, []);

    // 开始新任务时重置状态
    const startTask = useCallback((taskId: string, initialMessage: string) => {
        activeTaskIdRef.current = taskId;
        setModalOpen(true);
        setStatus('running');
        setPercent(0);
        setStatusText('正在准备导出...');
        setLogs([{
            time: new Date().toLocaleTimeString(),
            message: initialMessage,
            type: 'info',
        }]);
        setSummary([]);
    }, []);

    const finishTask = useCallback(() => {
        activeTaskIdRef.current = null;
    }, []);

    // 手动添加日志
    const addLog = useCallback((message: string, type: ProgressLog['type'] = 'info') => {
        setLogs(prev => [...prev, {
            time: new Date().toLocaleTimeString(),
            message,
            type,
        }]);
    }, []);

    return {
        modalOpen, setModalOpen,
        status, setStatus,
        percent, setPercent,
        statusText, setStatusText,
        logs,
        summary,
        startTask,
        finishTask,
        addLog,
    };
}
