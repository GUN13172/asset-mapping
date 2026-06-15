import React, { useState, useEffect, useRef } from 'react';
import { Card, Input, Button, Row, Col, Space, Typography, message, Select, Tag, Tooltip } from 'antd';
import {
    SendOutlined,
    ClearOutlined,
    CopyOutlined,
    ClockCircleOutlined,
    SwapOutlined,
    ThunderboltOutlined
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import { normalizeSmartPunctuation } from '../utils/textInput';

const { TextArea } = Input;
const { Text } = Typography;

interface RawHttpResponse {
    status: number;
    status_text: string;
    headers: Record<string, string>;
    body: string;
}

// 常用请求头预设
const commonHeaders: Record<string, string> = {
    'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    'Accept': '*/*',
    'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
    'Content-Type': 'application/x-www-form-urlencoded',
    'Connection': 'close',
};

interface ResenderProps {
    active?: boolean;
}

const Resender: React.FC<ResenderProps> = ({ active = true }) => {
    // 请求配置状态
    const [targetUrl, setTargetUrl] = useState<string>('');
    const [method, setMethod] = useState<string>('GET');
    const [request, setRequest] = useState<string>('GET / HTTP/1.1\nHost: example.com\nUser-Agent: Mozilla/5.0\nAccept: */*\n\n');
    const [response, setResponse] = useState<string>('');
    const [loading, setLoading] = useState<boolean>(false);
    const [responseTime, setResponseTime] = useState<number | null>(null);
    const [statusCode, setStatusCode] = useState<number | null>(null);
    const logRef = useRef<HTMLDivElement>(null);

    // 历史记录
    const [history, setHistory] = useState<{ url: string; method: string; status: number; time: number }[]>([]);

    // 检查是否有从资产查询页面发送过来的请求
    useEffect(() => {
        if (!active) return;

        const pendingRequest = localStorage.getItem('pending_resend_request');
        if (pendingRequest) {
            setRequest(normalizeSmartPunctuation(pendingRequest));
            localStorage.removeItem('pending_resend_request');
            // 尝试从请求中解析目标地址
            const hostMatch = pendingRequest.match(/Host:\s*(.+)/i);
            if (hostMatch) {
                const host = hostMatch[1].trim();
                setTargetUrl(`http://${host}`);
            }
            message.info('已加载来自资产查询的请求');
        }
    }, [active]);

    // 当目标 URL 改变时，自动更新请求中的 Host
    const handleUrlChange = (url: string) => {
        setTargetUrl(url);
        try {
            if (url) {
                const parsed = new URL(url.startsWith('http') ? url : `http://${url}`);
                const hostValue = parsed.port ? `${parsed.hostname}:${parsed.port}` : parsed.hostname;
                // 更新请求中的 Host 头
                setRequest(prev => {
                    if (prev.match(/Host:\s*.+/i)) {
                        return prev.replace(/Host:\s*.+/i, `Host: ${hostValue}`);
                    }
                    // 如果没有 Host 头，在第一行后添加
                    const lines = prev.split('\n');
                    lines.splice(1, 0, `Host: ${hostValue}`);
                    return lines.join('\n');
                });
            }
        } catch {
            // URL 解析失败时忽略
        }
    };

    // HTTP 方法切换时更新请求首行
    const handleMethodChange = (newMethod: string) => {
        setMethod(newMethod);
        setRequest(prev => {
            return prev.replace(/^(GET|POST|PUT|DELETE|PATCH|HEAD|OPTIONS)\s/i, `${newMethod} `);
        });
    };

    const handleSend = async () => {
        const normalizedRequest = normalizeSmartPunctuation(request);
        if (!normalizedRequest.trim()) {
            message.warning('请输入请求内容');
            return;
        }
        if (normalizedRequest !== request) {
            setRequest(normalizedRequest);
        }

        setLoading(true);
        setResponseTime(null);
        setStatusCode(null);
        const startTime = Date.now();

        try {
            const res: RawHttpResponse = await invoke('send_raw_http', { rawRequest: normalizedRequest });
            const elapsed = Date.now() - startTime;

            // 格式化响应输出
            let formattedResp = `HTTP/1.1 ${res.status} ${res.status_text}\n`;
            Object.entries(res.headers).forEach(([key, val]) => {
                formattedResp += `${key}: ${val}\n`;
            });
            formattedResp += '\n' + res.body;

            setResponse(formattedResp);
            setResponseTime(elapsed);
            setStatusCode(res.status);

            // 添加到历史
            setHistory(prev => [{
                url: targetUrl || '(raw)',
                method,
                status: res.status,
                time: elapsed
            }, ...prev.slice(0, 19)]);

            message.success(`请求完成 ${res.status} (${elapsed}ms)`);
        } catch (e: any) {
            const elapsed = Date.now() - startTime;
            setResponse(`[ERROR] 发送失败: ${e}`);
            setResponseTime(elapsed);
            setStatusCode(0);
            message.error(`发送失败: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    const handleCopy = () => {
        if (response) {
            navigator.clipboard.writeText(response);
            message.success('响应已复制到剪贴板');
        }
    };

    // 获取状态码对应的颜色
    const getStatusColor = (code: number) => {
        if (code >= 200 && code < 300) return '#52c41a';
        if (code >= 300 && code < 400) return '#faad14';
        if (code >= 400 && code < 500) return '#ff4d4f';
        if (code >= 500) return '#ff006e';
        return '#d9d9d9';
    };

    // 快捷插入请求头
    const insertHeader = (key: string, value: string) => {
        setRequest(prev => {
            const lines = prev.split('\n');
            // 找到空行（请求体之前）
            const emptyLineIdx = lines.findIndex((l, i) => i > 0 && l.trim() === '');
            if (emptyLineIdx > 0) {
                lines.splice(emptyLineIdx, 0, `${key}: ${value}`);
            } else {
                lines.push(`${key}: ${value}`);
            }
            return lines.join('\n');
        });
    };

    return (
        <div className="resender-container fade-in">
            {/* 顶部工具栏 */}
            <Card className="glass-effect" bordered={false} bodyStyle={{ padding: '12px 20px' }} style={{ marginBottom: 16 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                    <Select
                        value={method}
                        onChange={handleMethodChange}
                        style={{ width: 120 }}
                        size="large"
                    >
                        {['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'].map(m => (
                            <Select.Option key={m} value={m}>
                                <Tag color={
                                    m === 'GET' ? 'green' : m === 'POST' ? 'orange' :
                                        m === 'PUT' ? 'blue' : m === 'DELETE' ? 'red' :
                                            m === 'PATCH' ? 'purple' : 'default'
                                } style={{ margin: 0 }}>{m}</Tag>
                            </Select.Option>
                        ))}
                    </Select>
                    <Input
                        value={targetUrl}
                        onChange={e => handleUrlChange(e.target.value)}
                        placeholder="输入目标 URL，如 https://example.com"
                        size="large"
                        style={{ flex: 1 }}
                        className="glass-effect"
                        prefix={<ThunderboltOutlined style={{ color: 'var(--accent-cyan)' }} />}
                        onPressEnter={handleSend}
                    />
                    <Button
                        type="primary"
                        size="large"
                        icon={<SendOutlined />}
                        onClick={handleSend}
                        loading={loading}
                        className="gradient-button"
                        style={{ width: 120 }}
                    >
                        发送
                    </Button>
                </div>

                {/* 状态信息栏 */}
                {statusCode !== null && (
                    <div style={{
                        marginTop: 10,
                        display: 'flex',
                        gap: 16,
                        alignItems: 'center',
                        padding: '6px 0'
                    }}>
                        <Tag
                            color={getStatusColor(statusCode)}
                            style={{ fontSize: '14px', padding: '2px 12px' }}
                        >
                            {statusCode > 0 ? `${statusCode}` : 'Error'}
                        </Tag>
                        {responseTime !== null && (
                            <Space size={4}>
                                <ClockCircleOutlined style={{ color: 'var(--text-muted)' }} />
                                <Text type="secondary">
                                    {responseTime < 1000 ? `${responseTime} ms` : `${(responseTime / 1000).toFixed(2)} s`}
                                </Text>
                            </Space>
                        )}
                        {response && (
                            <Text type="secondary" style={{ fontSize: '12px' }}>
                                响应大小: {new Blob([response]).size.toLocaleString()} bytes
                            </Text>
                        )}
                    </div>
                )}
            </Card>

            <Row gutter={16}>
                {/* 请求面板 */}
                <Col span={12}>
                    <Card
                        title={
                            <Space>
                                <SendOutlined style={{ color: 'var(--accent-cyan)' }} />
                                <span>请求 (Request)</span>
                            </Space>
                        }
                        extra={
                            <Space>
                                <Tooltip title="快捷插入请求头">
                                    <Select
                                        placeholder="插入请求头"
                                        size="small"
                                        style={{ width: 140 }}
                                        onChange={(key: string) => {
                                            insertHeader(key, commonHeaders[key]);
                                        }}
                                        value={undefined}
                                    >
                                        {Object.keys(commonHeaders).map(h => (
                                            <Select.Option key={h} value={h}>{h}</Select.Option>
                                        ))}
                                    </Select>
                                </Tooltip>
                                <Button size="small" icon={<ClearOutlined />} onClick={() => {
                                    setRequest(`${method} / HTTP/1.1\nHost: \nUser-Agent: Mozilla/5.0\nAccept: */*\n\n`);
                                }}>重置</Button>
                            </Space>
                        }
                        className="glass-effect"
                        bordered={false}
                        style={{ height: 'calc(100vh - 260px)' }}
                    >
                        <TextArea
                            value={request}
                            onChange={(e) => setRequest(normalizeSmartPunctuation(e.target.value))}
                            style={{
                                height: '100%',
                                fontFamily: "'Consolas', 'Monaco', 'Fira Code', monospace",
                                background: 'rgba(0,0,0,0.3)',
                                color: '#00ff88',
                                border: '1px solid var(--border-color)',
                                borderRadius: '8px',
                                resize: 'none',
                                fontSize: '13px',
                                lineHeight: '1.6',
                                padding: '12px',
                            }}
                            rows={25}
                            autoCorrect="off"
                            autoCapitalize="off"
                            autoComplete="off"
                            spellCheck={false}
                        />
                    </Card>
                </Col>

                {/* 响应面板 */}
                <Col span={12}>
                    <Card
                        title={
                            <Space>
                                <SwapOutlined style={{ color: 'var(--accent-green)' }} />
                                <span>响应 (Response)</span>
                            </Space>
                        }
                        extra={
                            <Space>
                                <Button size="small" icon={<CopyOutlined />} onClick={handleCopy}
                                    disabled={!response}>复制</Button>
                                <Button size="small" icon={<ClearOutlined />} onClick={() => {
                                    setResponse('');
                                    setStatusCode(null);
                                    setResponseTime(null);
                                }} disabled={!response}>清空</Button>
                            </Space>
                        }
                        className="glass-effect"
                        bordered={false}
                        style={{ height: 'calc(100vh - 260px)' }}
                    >
                        <div
                            ref={logRef}
                            style={{
                                height: '100%',
                                overflow: 'auto',
                                padding: '12px',
                                background: 'rgba(0,0,0,0.3)',
                                borderRadius: '8px',
                                fontFamily: "'Consolas', 'Monaco', 'Fira Code', monospace",
                                fontSize: '13px',
                                lineHeight: '1.6',
                                border: '1px solid var(--border-color)',
                                whiteSpace: 'pre-wrap',
                            }}
                        >
                            {response ? (
                                response.split('\n').map((line, i) => {
                                    // 响应状态行高亮
                                    if (i === 0 && line.startsWith('HTTP/')) {
                                        const statusMatch = line.match(/(\d{3})/);
                                        const code = statusMatch ? parseInt(statusMatch[1]) : 0;
                                        return (
                                            <div key={i} style={{ color: getStatusColor(code), fontWeight: 'bold' }}>
                                                {line}
                                            </div>
                                        );
                                    }
                                    // 响应头高亮
                                    if (line.includes(':') && !line.startsWith(' ') && !line.startsWith('\t') && i < 50 && response.indexOf('\n\n') > response.indexOf(line)) {
                                        const colonIdx = line.indexOf(':');
                                        return (
                                            <div key={i}>
                                                <span style={{ color: '#569cd6' }}>{line.substring(0, colonIdx)}</span>
                                                <span style={{ color: '#d4d4d4' }}>:</span>
                                                <span style={{ color: '#ce9178' }}>{line.substring(colonIdx + 1)}</span>
                                            </div>
                                        );
                                    }
                                    // 错误信息
                                    if (line.startsWith('[ERROR]')) {
                                        return <div key={i} style={{ color: '#ff4d4f' }}>{line}</div>;
                                    }
                                    // 普通内容
                                    return <div key={i} style={{ color: '#d4d4d4' }}>{line || '\u00A0'}</div>;
                                })
                            ) : (
                                <div style={{
                                    display: 'flex',
                                    flexDirection: 'column',
                                    alignItems: 'center',
                                    justifyContent: 'center',
                                    height: '100%',
                                    opacity: 0.4,
                                    gap: 12,
                                }}>
                                    <SwapOutlined style={{ fontSize: 48, color: 'var(--text-muted)' }} />
                                    <Text type="secondary">发送请求后在此处查看响应</Text>
                                </div>
                            )}
                        </div>
                    </Card>
                </Col>
            </Row>

            {/* 请求历史（底部小条） */}
            {history.length > 0 && (
                <Card className="glass-effect" bordered={false} bodyStyle={{ padding: '8px 16px' }} style={{ marginTop: 12 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, overflowX: 'auto' }}>
                        <Text type="secondary" style={{ fontSize: '12px', whiteSpace: 'nowrap' }}>最近请求:</Text>
                        {history.slice(0, 8).map((h, i) => (
                            <Tag
                                key={i}
                                color={getStatusColor(h.status)}
                                style={{ cursor: 'pointer', fontSize: '11px', margin: 0 }}
                                onClick={() => {
                                    setTargetUrl(h.url);
                                    setMethod(h.method);
                                }}
                            >
                                {h.method} {h.status} ({h.time}ms)
                            </Tag>
                        ))}
                    </div>
                </Card>
            )}
        </div>
    );
};

export default Resender;
