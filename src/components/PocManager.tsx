import React, { useState, useMemo } from 'react';
import { Card, Table, Tag, Input, Space, Button, Typography, Layout, Menu, Drawer, Badge, Divider, message } from 'antd';
import {
    DatabaseOutlined,
    SearchOutlined,
    EyeOutlined,
    FireOutlined,
    TagsOutlined,
    UserOutlined,
    CopyOutlined,
    PlayCircleOutlined,
    SyncOutlined,
    FolderOpenOutlined,
    RocketOutlined
} from '@ant-design/icons';
import type { MenuProps } from 'antd';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';

const { Text, Title } = Typography;
const { Sider, Content } = Layout;

type MenuItem = Required<MenuProps>['items'][number];

interface PocTemplate {
    id: string;
    name: string;
    description: string;
    severity: string;
    author: string;
    tags: string[];
    content: string;
    path: string;
}

const PocManager: React.FC = () => {
    const [searchText, setSearchText] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('all');
    const [selectedPoc, setSelectedPoc] = useState<PocTemplate | null>(null);
    const [drawerVisible, setDrawerVisible] = useState(false);
    const [pocs, setPocs] = useState<PocTemplate[]>([]);
    const [loading, setLoading] = useState(false);

    const fetchPocs = async () => {
        setLoading(true);
        try {
            const data = await invoke<PocTemplate[]>('list_pocs');
            setPocs(data || []);
        } catch (e: any) {
            message.error(`获取模版失败: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    React.useEffect(() => {
        fetchPocs();
    }, []);

    const handlePullLatest = async () => {
        setLoading(true);
        message.loading({ content: '正在检查模版更新...', key: 'pull' });
        try {
            const res = await invoke<string>('pull_latest_pocs');
            message.success({ content: `更新成功: ${res}`, key: 'pull', duration: 4 });
            fetchPocs();
        } catch (e: any) {
            message.error({ content: `更新失败: ${e}`, key: 'pull', duration: 4 });
        } finally {
            setLoading(false);
        }
    };

    const handleImportLocal = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: '选择 POC 模版目录'
            });
            if (selected) {
                setLoading(true);
                const results = await invoke<PocTemplate[]>('import_local_pocs', { path: selected });
                setPocs(results);
                message.success(`成功从 ${selected} 导入 ${results.length} 个模版`);
            }
        } catch (e: any) {
            message.error(`导入失败: ${e}`);
        } finally {
            setLoading(false);
        }
    };

    // 计算统计数据
    const stats = useMemo(() => {
        const counts = {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
            all: pocs.length
        };
        pocs.forEach(p => {
            const s = p.severity.toLowerCase();
            if (s in counts) counts[s as keyof typeof counts]++;
        });
        return counts;
    }, [pocs]);

    // 过滤逻辑
    const filteredPocs = useMemo(() => {
        return pocs.filter(p => {
            const matchesSearch = p.name.toLowerCase().includes(searchText.toLowerCase()) ||
                p.tags.some(t => t.toLowerCase().includes(searchText.toLowerCase()));
            const matchesCategory = selectedCategory === 'all' ||
                p.severity.toLowerCase() === selectedCategory.toLowerCase() ||
                p.author.toLowerCase() === selectedCategory.toLowerCase();
            return matchesSearch && matchesCategory;
        });
    }, [pocs, searchText, selectedCategory]);

    const columns = [
        {
            title: '名称',
            dataIndex: 'name',
            key: 'name',
            width: '40%',
            render: (text: string, record: PocTemplate) => (
                <div style={{ padding: '4px 0' }}>
                    <div style={{ color: 'var(--accent-cyan)', fontWeight: 600, fontSize: '14px', lineHeight: '1.5' }}>{text}</div>
                    <div style={{
                        fontSize: '12px',
                        color: 'rgba(255,255,255,0.45)',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        marginTop: '2px'
                    }} title={record.description}>
                        {record.description}
                    </div>
                </div>
            )
        },
        {
            title: '危害',
            dataIndex: 'severity',
            key: 'severity',
            width: 120,
            render: (severity: string) => {
                const s = severity.toLowerCase();
                const colors = { critical: '#eb2f96', high: '#f5222d', medium: '#fa8c16', low: '#1890ff', info: '#8c8c8c' };
                const color = colors[s as keyof typeof colors] || colors.info;
                return <Badge color={color} text={<span style={{ fontSize: '12px' }}>{severity.toUpperCase()}</span>} />;
            }
        },
        {
            title: '标签',
            dataIndex: 'tags',
            key: 'tags',
            width: '20%',
            render: (tags: string[]) => (
                <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                    {tags.slice(0, 2).map(tag => (
                        <Tag key={tag} bordered={false} style={{ background: 'rgba(255,255,255,0.05)', fontSize: '10px', margin: 0 }}>
                            {tag}
                        </Tag>
                    ))}
                    {tags.length > 2 && <span style={{ fontSize: '10px', color: 'rgba(255,255,255,0.3)' }}>+{tags.length - 2}</span>}
                </div>
            )
        },
        {
            title: '作者',
            dataIndex: 'author',
            key: 'author',
            width: 120,
            render: (author: string) => (
                <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                    <UserOutlined style={{ fontSize: '12px', color: 'rgba(255,255,255,0.3)' }} />
                    <span style={{ fontSize: '13px', opacity: 0.8 }}>{author}</span>
                </div>
            )
        },
        {
            title: '操作',
            key: 'action',
            width: 130,
            fixed: 'right' as const,
            render: (_: any, record: PocTemplate) => (
                <Space size={0}>
                    <Button
                        size="small"
                        type="link"
                        icon={<EyeOutlined />}
                        onClick={() => { setSelectedPoc(record); setDrawerVisible(true); }}
                        style={{ color: 'var(--accent-cyan)', padding: '0 8px' }}
                    >
                        查看
                    </Button>
                    <Button
                        size="small"
                        type="link"
                        icon={<PlayCircleOutlined />}
                        style={{ color: 'var(--accent-green)', padding: '0 8px' }}
                        onClick={() => message.info('正在开发中...')}
                    >
                        验证
                    </Button>
                </Space>
            ),
        },
    ];

    const menuItems: MenuItem[] = [
        { key: 'all', icon: <DatabaseOutlined />, label: `全部模板 (${stats.all})` },
        { type: 'divider' },
        { key: 'critical', icon: <FireOutlined style={{ color: '#eb2f96' }} />, label: `严重冲击 (${stats.critical})` },
        { key: 'high', icon: <FireOutlined style={{ color: '#f5222d' }} />, label: `高危漏洞 (${stats.high})` },
        { key: 'medium', icon: <FireOutlined style={{ color: '#fa8c16' }} />, label: `中危风险 (${stats.medium})` },
    ];

    return (
        <div className="poc-manager-container fade-in" style={{ height: 'calc(100vh - 120px)', display: 'flex' }}>
            <Layout style={{ background: 'transparent', height: '100%', width: '100%' }}>
                <Sider
                    width={200}
                    className="glass-effect"
                    style={{ borderRadius: '12px', marginRight: '16px', padding: '8px 0', border: '1px solid var(--border-color)' }}
                >
                    <div style={{ padding: '8px 16px', marginBottom: 8 }}>
                        <Text strong style={{ opacity: 0.7 }}>分类筛选</Text>
                    </div>
                    <Menu
                        mode="inline"
                        selectedKeys={[selectedCategory]}
                        onClick={({ key }) => setSelectedCategory(key)}
                        items={menuItems}
                        style={{ background: 'transparent', borderRight: 0 }}
                    />
                </Sider>
                <Content style={{ height: '100%' }}>
                    <Card
                        title={
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', width: '100%' }}>
                                <Space>
                                    <DatabaseOutlined style={{ color: 'var(--accent-cyan)' }} />
                                    <span style={{ fontWeight: 600 }}>POC 模板库</span>
                                    <Tag color="cyan" bordered={false} style={{ borderRadius: '10px' }}>{filteredPocs.length} 个模版</Tag>
                                </Space>
                                <Space>
                                    <Button
                                        type="default"
                                        icon={<RocketOutlined />}
                                        onClick={() => {
                                            const targets = filteredPocs.map(p => ({ name: p.name, path: p.path })); // Pass path and name
                                            if (targets.length === 0) {
                                                message.warning('当前列表为空，无可用模板');
                                                return;
                                            }
                                            localStorage.setItem('selected_pocs', JSON.stringify(targets));
                                            message.success({
                                                content: `已加载 ${targets.length} 个模板到扫描引擎，请前往“漏洞扫描”页面执行`,
                                                duration: 5
                                            });
                                        }}
                                        className="glass-button"
                                        style={{ marginRight: 8 }}
                                    >
                                        验证当前 ({filteredPocs.length})
                                    </Button>
                                    <Button
                                        icon={<SyncOutlined spin={loading} />}
                                        onClick={handlePullLatest}
                                        loading={loading}
                                        className="glass-button"
                                    >
                                        拉取最新
                                    </Button>
                                    <Button
                                        type="primary"
                                        icon={<FolderOpenOutlined />}
                                        onClick={handleImportLocal}
                                        loading={loading}
                                        style={{ borderRadius: '8px' }}
                                    >
                                        本地扫描导入
                                    </Button>
                                </Space>
                            </div>
                        }
                        className="glass-effect"
                        bodyStyle={{ padding: 0, height: 'calc(100% - 58px)', overflow: 'hidden' }}
                        style={{ height: '100%', border: '1px solid var(--border-color)' }}
                    >
                        <div style={{ padding: '12px 16px', borderBottom: '1px solid var(--border-color)' }}>
                            <Input
                                prefix={<SearchOutlined style={{ opacity: 0.5 }} />}
                                placeholder="搜索 POC 名称、作者、描述或标签..."
                                style={{ width: '100%' }}
                                bordered={false}
                                value={searchText}
                                onChange={e => setSearchText(e.target.value)}
                                className="glass-input"
                                allowClear
                            />
                        </div>
                        <Table
                            columns={columns}
                            dataSource={filteredPocs}
                            rowKey="id"
                            loading={loading}
                            size="small"
                            scroll={{ y: 'calc(100vh - 350px)' }}
                            pagination={{
                                pageSize: 15,
                                showTotal: (total) => `共 ${total} 条模版`,
                                size: 'small',
                                style: { padding: '10px 16px', margin: 0 }
                            }}
                            className="poc-table"
                        />
                    </Card>
                </Content>
            </Layout>

            <Drawer
                title={
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span>模版详情: {selectedPoc?.name}</span>
                        <Space>
                            <Button size="small" icon={<CopyOutlined />}>复制 YAML</Button>
                            <Button size="small" type="primary" icon={<PlayCircleOutlined />}>立即执行</Button>
                        </Space>
                    </div>
                }
                placement="right"
                width={700}
                onClose={() => setDrawerVisible(false)}
                open={drawerVisible}
                extra={
                    <Space>
                        {selectedPoc?.severity && (
                            <Tag color={selectedPoc.severity === 'critical' ? 'magenta' : 'red'}>
                                {selectedPoc.severity.toUpperCase()}
                            </Tag>
                        )}
                    </Space>
                }
                maskStyle={{ backdropFilter: 'blur(4px)' }}
                headerStyle={{ background: 'var(--bg-secondary)', borderBottom: '1px solid var(--border-color)' }}
                bodyStyle={{ background: 'var(--bg-primary)', padding: 0 }}
            >
                {selectedPoc && (
                    <div style={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
                        <div style={{ padding: '20px', background: 'rgba(255,255,255,0.02)' }}>
                            <Title level={4}>{selectedPoc.name}</Title>
                            <Text type="secondary">{selectedPoc.description}</Text>
                            <div style={{ marginTop: 16 }}>
                                <Space wrap>
                                    <Tag icon={<UserOutlined />}>{selectedPoc.author}</Tag>
                                    {selectedPoc.tags.map(t => <Tag key={t} icon={<TagsOutlined />}>{t}</Tag>)}
                                </Space>
                            </div>
                        </div>
                        <Divider style={{ margin: 0 }} />
                        <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
                            <Title level={5}>YAML 内容</Title>
                            <pre style={{
                                background: '#1e1e1e',
                                color: '#d4d4d4',
                                padding: '16px',
                                borderRadius: '8px',
                                fontFamily: 'Consolas, Monaco, monospace',
                                fontSize: '13px',
                                border: '1px solid var(--border-color)',
                                whiteSpace: 'pre-wrap'
                            }}>
                                {selectedPoc.content}
                            </pre>
                        </div>
                    </div>
                )}
            </Drawer>
        </div>
    );
};

export default PocManager;
