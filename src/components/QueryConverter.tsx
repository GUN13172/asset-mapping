import React, { useState, useEffect } from 'react';
import { 
  Card, 
  Input, 
  Select, 
  Button, 
  Space, 
  Typography, 
  message,
  Alert,
  Row,
  Col,
  Divider,
  Tooltip,
  Tag
} from 'antd';
import {
  SwapOutlined,
  CopyOutlined,
  CheckCircleOutlined,
  CloseCircleOutlined,
  ReloadOutlined,
  InfoCircleOutlined
} from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';

const { TextArea } = Input;
const { Title, Text, Paragraph } = Typography;
const { Option } = Select;

interface ConversionResult {
  platform: string;
  query: string;
}

const QueryConverter: React.FC = () => {
  const [query, setQuery] = useState('');
  const [fromPlatform, setFromPlatform] = useState<string>('fofa');
  const [toPlatform, setToPlatform] = useState<string>('');
  const [platforms, setPlatforms] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [validating, setValidating] = useState(false);
  const [converting, setConverting] = useState(false);
  const [conversionResults, setConversionResults] = useState<ConversionResult[]>([]);
  const [validationResult, setValidationResult] = useState<{ valid: boolean; error?: string } | null>(null);
  const [conversionMode, setConversionMode] = useState<'single' | 'all'>('all');

  // 平台展示名称映射
  const platformNames: Record<string, string> = {
    fofa: 'FOFA',
    quake: 'QUAKE',
    hunter: 'Hunter',
    daydaymap: 'DayDayMap'
  };

  // 平台颜色映射
  const platformColors: Record<string, string> = {
    fofa: 'blue',
    quake: 'purple',
    hunter: 'orange',
    daydaymap: 'cyan'
  };

  // 加载支持的平台列表
  useEffect(() => {
    loadPlatforms();
  }, []);

  const loadPlatforms = async () => {
    try {
      setLoading(true);
      const result = await invoke<string[]>('get_supported_platforms');
      setPlatforms(result);
      if (result.length > 0 && !fromPlatform) {
        setFromPlatform(result[0]);
      }
    } catch (error) {
      message.error(`加载平台列表失败: ${error}`);
    } finally {
      setLoading(false);
    }
  };

  // 验证查询语法
  const handleValidate = async () => {
    if (!query.trim()) {
      message.warning('请输入查询语句');
      return;
    }

    try {
      setValidating(true);
      await invoke('validate_query_syntax', {
        query: query.trim(),
        platform: fromPlatform
      });
      setValidationResult({ valid: true });
      message.success('查询语法验证通过！');
    } catch (error) {
      setValidationResult({ valid: false, error: String(error) });
      message.error(`语法验证失败: ${error}`);
    } finally {
      setValidating(false);
    }
  };

  // 转换查询语句
  const handleConvert = async () => {
    if (!query.trim()) {
      message.warning('请输入查询语句');
      return;
    }

    if (conversionMode === 'single' && !toPlatform) {
      message.warning('请选择目标平台');
      return;
    }

    try {
      setConverting(true);
      setConversionResults([]);

      if (conversionMode === 'all') {
        // 转换到所有平台
        const results = await invoke<ConversionResult[]>('convert_query_to_all', {
          query: query.trim(),
          fromPlatform: fromPlatform
        });
        setConversionResults(results);
        message.success(`成功转换到 ${results.length} 个平台！`);
      } else {
        // 转换到指定平台
        const result = await invoke<string>('convert_query', {
          query: query.trim(),
          fromPlatform: fromPlatform,
          toPlatform: toPlatform
        });
        setConversionResults([{ platform: toPlatform, query: result }]);
        message.success('转换成功！');
      }
    } catch (error) {
      message.error(`转换失败: ${error}`);
    } finally {
      setConverting(false);
    }
  };

  // 复制到剪贴板
  const handleCopy = (text: string, platform: string) => {
    navigator.clipboard.writeText(text).then(() => {
      message.success(`已复制 ${platformNames[platform] || platform} 查询语句`);
    }).catch(() => {
      message.error('复制失败');
    });
  };

  // 重置表单
  const handleReset = () => {
    setQuery('');
    setConversionResults([]);
    setValidationResult(null);
  };

  // 示例查询语句
  const exampleQueries: Record<string, string[]> = {
    fofa: [
      'ip="8.8.8.8"',
      'title="登录" && country="CN"',
      'body="powered by" && port="80"'
    ],
    quake: [
      'ip:"8.8.8.8"',
      'title:"登录" AND country:"CN"',
      'body:"powered by" AND port:"80"'
    ],
    hunter: [
      'ip="8.8.8.8"',
      'web.title="登录" && country="CN"',
      'web.body="powered by" && ip.port="80"'
    ],
    daydaymap: [
      'ip="8.8.8.8"',
      'title="登录" && country="CN"',
      'body="powered by" && port="80"'
    ]
  };

  const loadExample = (index: number) => {
    const examples = exampleQueries[fromPlatform] || exampleQueries['fofa'];
    if (examples[index]) {
      setQuery(examples[index]);
      setValidationResult(null);
      setConversionResults([]);
    }
  };

  return (
    <div style={{ padding: '0 24px' }}>
      <Space direction="vertical" size="large" style={{ width: '100%' }}>
        {/* 标题和说明 */}
        <div>
          <Title level={2}>
            <SwapOutlined style={{ marginRight: 8 }} />
            查询语句转换
          </Title>
          <Paragraph type="secondary">
            支持在 FOFA、QUAKE、Hunter、DayDayMap 等测绘平台的查询语句之间互相转换
          </Paragraph>
        </div>

        {/* 输入区域 */}
        <Card title="输入查询语句" loading={loading}>
          <Space direction="vertical" size="middle" style={{ width: '100%' }}>
            <Row gutter={16}>
              <Col span={12}>
                <Text strong>源平台：</Text>
                <Select
                  value={fromPlatform}
                  onChange={(value) => {
                    setFromPlatform(value);
                    setValidationResult(null);
                    setConversionResults([]);
                  }}
                  style={{ width: '100%', marginTop: 8 }}
                  size="large"
                >
                  {platforms.map(platform => (
                    <Option key={platform} value={platform}>
                      <Tag color={platformColors[platform]}>{platformNames[platform] || platform}</Tag>
                    </Option>
                  ))}
                </Select>
              </Col>
              <Col span={12}>
                <Text strong>转换模式：</Text>
                <Select
                  value={conversionMode}
                  onChange={setConversionMode}
                  style={{ width: '100%', marginTop: 8 }}
                  size="large"
                >
                  <Option value="all">转换到所有平台</Option>
                  <Option value="single">转换到指定平台</Option>
                </Select>
              </Col>
            </Row>

            {conversionMode === 'single' && (
              <div>
                <Text strong>目标平台：</Text>
                <Select
                  value={toPlatform}
                  onChange={setToPlatform}
                  style={{ width: '100%', marginTop: 8 }}
                  size="large"
                  placeholder="选择目标平台"
                >
                  {platforms.filter(p => p !== fromPlatform).map(platform => (
                    <Option key={platform} value={platform}>
                      <Tag color={platformColors[platform]}>{platformNames[platform] || platform}</Tag>
                    </Option>
                  ))}
                </Select>
              </div>
            )}

            <div>
              <Space style={{ marginBottom: 8 }}>
                <Text strong>查询语句：</Text>
                <Tooltip title="点击加载示例">
                  <Button type="link" size="small" onClick={() => loadExample(0)}>示例1</Button>
                  <Button type="link" size="small" onClick={() => loadExample(1)}>示例2</Button>
                  <Button type="link" size="small" onClick={() => loadExample(2)}>示例3</Button>
                </Tooltip>
              </Space>
              <TextArea
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  setValidationResult(null);
                }}
                placeholder="输入查询语句，例如: ip=&quot;8.8.8.8&quot; && port=&quot;80&quot;"
                rows={4}
                style={{ fontFamily: 'monospace' }}
                autoCorrect="off"
                autoCapitalize="off"
                spellCheck={false}
                autoComplete="off"
              />
            </div>

            {/* 验证结果 */}
            {validationResult && (
              <Alert
                message={validationResult.valid ? '语法验证通过' : '语法验证失败'}
                description={!validationResult.valid ? validationResult.error : undefined}
                type={validationResult.valid ? 'success' : 'error'}
                icon={validationResult.valid ? <CheckCircleOutlined /> : <CloseCircleOutlined />}
                showIcon
                closable
                onClose={() => setValidationResult(null)}
              />
            )}

            {/* 操作按钮 */}
            <Row gutter={16}>
              <Col span={8}>
                <Button
                  type="default"
                  icon={<CheckCircleOutlined />}
                  onClick={handleValidate}
                  loading={validating}
                  block
                  size="large"
                >
                  验证语法
                </Button>
              </Col>
              <Col span={8}>
                <Button
                  type="primary"
                  icon={<SwapOutlined />}
                  onClick={handleConvert}
                  loading={converting}
                  block
                  size="large"
                >
                  开始转换
                </Button>
              </Col>
              <Col span={8}>
                <Button
                  icon={<ReloadOutlined />}
                  onClick={handleReset}
                  block
                  size="large"
                >
                  重置
                </Button>
              </Col>
            </Row>
          </Space>
        </Card>

        {/* 转换结果 */}
        {conversionResults.length > 0 && (
          <Card title="转换结果" extra={<Text type="secondary">{conversionResults.length} 个平台</Text>}>
            <Space direction="vertical" size="middle" style={{ width: '100%' }}>
              {conversionResults.map((result) => (
                <Card
                  key={result.platform}
                  type="inner"
                  title={
                    <Space>
                      <Tag color={platformColors[result.platform]}>
                        {platformNames[result.platform] || result.platform}
                      </Tag>
                    </Space>
                  }
                  extra={
                    <Button
                      type="link"
                      icon={<CopyOutlined />}
                      onClick={() => handleCopy(result.query, result.platform)}
                    >
                      复制
                    </Button>
                  }
                >
                  <TextArea
                    value={result.query}
                    readOnly
                    autoSize={{ minRows: 2, maxRows: 6 }}
                    style={{ 
                      fontFamily: 'monospace',
                      backgroundColor: '#f5f5f5'
                    }}
                    autoCorrect="off"
                    autoCapitalize="off"
                    spellCheck={false}
                  />
                </Card>
              ))}
            </Space>
          </Card>
        )}

        {/* 帮助信息 */}
        <Card title={<><InfoCircleOutlined style={{ marginRight: 8 }} />使用说明</>}>
          <Space direction="vertical" size="small">
            <Text>1. 选择源平台并输入查询语句</Text>
            <Text>2. 选择转换模式（转换到所有平台或指定平台）</Text>
            <Text>3. 点击"验证语法"检查语句是否正确（可选）</Text>
            <Text>4. 点击"开始转换"执行转换</Text>
            <Text>5. 在转换结果中查看并复制转换后的查询语句</Text>
            <Divider />
            <Text type="secondary" style={{ fontSize: '12px' }}>
              注意：不同平台支持的字段和操作符可能有所不同，转换结果仅供参考
            </Text>
          </Space>
        </Card>
      </Space>
    </div>
  );
};

export default QueryConverter;
