import React, { useState } from 'react';
import { Card, Input, Button, Row, Col, Space, Typography, message } from 'antd';
import { SendOutlined, ClearOutlined, CopyOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';

const { TextArea } = Input;
const { Text } = Typography;

interface RawHttpResponse {
    status: number;
    status_text: String;
    headers: Record<string, string>;
    body: string;
}

const Resender: React.FC = () => {
    const [request, setRequest] = useState<string>('GET / HTTP/1.1\nHost: example.com\nUser-Agent: Mozilla/5.0\nAccept: */*\n\n');
    const [response, setResponse] = useState<string>('');
    const [loading, setLoading] = useState<boolean>(false);

    const handleSend = async () => {
        if (!request.trim()) {
            message.warning('请输入请求内容');
            return;
        }

        setLoading(true);
        try {
            const res: RawHttpResponse = await invoke('send_raw_http', { rawRequest: request });

            // 格式化响应输出
            let formattedResp = `HTTP/1.1 ${res.status} ${res.status_text}\n`;
            Object.entries(res.headers).forEach(([key, val]) => {
                formattedResp += `${key}: ${val}\n`;
            });
            formattedResp += '\n' + res.body;

            setResponse(formattedResp);
            message.success('请求发送成功');
        } catch (e: any) {
            setResponse(`[ERROR] 发送失败: ${e}`);
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

    return (
        <div className="resender-container fade-in">
            <Row gutter={16}>
                <Col span={12}>
                    <Card
                        title={
                            <Space>
                                <SendOutlined />
                                <span>请求 (Request)</span>
                            </Space>
                        }
                        extra={
                            <Space>
                                <Button size="small" icon={<ClearOutlined />} onClick={() => setRequest('')}>清空</Button>
                                <Button type="primary" size="small" icon={<SendOutlined />} onClick={handleSend} loading={loading}>发送</Button>
                            </Space>
                        }
                        className="glass-effect"
                        style={{ height: 'calc(100vh - 160px)' }}
                    >
                        <TextArea
                            value={request}
                            onChange={(e) => setRequest(e.target.value)}
                            style={{
                                height: '100%',
                                fontFamily: "'Consolas', 'Monaco', monospace",
                                background: 'rgba(0,0,0,0.2)',
                                color: '#00ff00',
                                border: 'none',
                                resize: 'none',
                                fontSize: '13px'
                            }}
                            rows={25}
                        />
                    </Card>
                </Col>
                <Col span={12}>
                    <Card
                        title="响应 (Response)"
                        extra={<Button size="small" icon={<CopyOutlined />} onClick={handleCopy}>复制</Button>}
                        className="glass-effect"
                        style={{ height: 'calc(100vh - 160px)' }}
                    >
                        <div style={{
                            height: '100%',
                            overflow: 'auto',
                            padding: '12px',
                            background: 'rgba(0,0,0,0.3)',
                            borderRadius: '4px',
                            fontFamily: "'Consolas', 'Monaco', monospace",
                            color: '#ccc',
                            whiteSpace: 'pre-wrap',
                            fontSize: '13px'
                        }}>
                            {response || <Text type="secondary">等待响应数据...</Text>}
                        </div>
                    </Card>
                </Col>
            </Row>
        </div>
    );
};

export default Resender;
