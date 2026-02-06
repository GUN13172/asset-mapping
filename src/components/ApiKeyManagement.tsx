import React, { useState, useEffect } from 'react';
import { Card, Tabs, Input, Button, Table, Form, Modal, message, Popconfirm, Tag, Space } from 'antd';
import { PlusOutlined, DeleteOutlined, CheckCircleOutlined, ExclamationCircleOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';

interface ApiKey {
  key: string;
  email?: string;
  status?: 'valid' | 'invalid' | 'unknown';
  quota?: string;
}

const ApiKeyManagement: React.FC = () => {
  const [platform, setPlatform] = useState<string>('hunter');
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [loading, setLoading] = useState<boolean>(false);
  const [isModalVisible, setIsModalVisible] = useState<boolean>(false);
  const [isBatchMode, setIsBatchMode] = useState<boolean>(false);
  const [form] = Form.useForm();

  // 加载API密钥
  useEffect(() => {
    fetchApiKeys();
  }, [platform]);

  const fetchApiKeys = async () => {
    setLoading(true);
    try {
      const result = await invoke('get_api_keys', { platform });
      const data = result as { api_keys: string[], emails?: string[] };
      
      let keys: ApiKey[] = [];
      
      if (platform === 'fofa' && data.emails) {
        // FOFA平台需要同时处理API密钥和邮箱
        keys = data.api_keys.map((key, index) => ({
          key,
          email: data.emails?.[index] || '',
          status: 'unknown'
        }));
      } else {
        // 其他平台只处理API密钥
        keys = data.api_keys.map(key => ({
          key,
          status: 'unknown'
        }));
      }
      
      setApiKeys(keys);
    } catch (error) {
      console.error('获取API密钥出错:', error);
      message.error('获取API密钥失败');
    } finally {
      setLoading(false);
    }
  };

  // 添加API密钥
  const addApiKey = async (values: { apiKey: string, email?: string, batchKeys?: string }) => {
    try {
      // 批量添加模式
      if (isBatchMode && values.batchKeys) {
        const keys = values.batchKeys
          .split('\n')
          .map(k => k.trim())
          .filter(k => k.length > 0);
        
        if (keys.length === 0) {
          message.warning('请输入至少一个API密钥');
          return;
        }

        let successCount = 0;
        let failCount = 0;

        message.loading('正在批量添加API密钥...', 0);

        for (const key of keys) {
          try {
            if (platform === 'fofa' && values.email) {
              await invoke('add_api_key', { 
                platform, 
                apiKey: key,
                email: values.email 
              });
            } else {
              await invoke('add_api_key', { 
                platform, 
                apiKey: key 
              });
            }
            successCount++;
          } catch (error) {
            console.error(`添加密钥 ${key} 失败:`, error);
            failCount++;
          }
        }

        message.destroy();
        
        if (failCount === 0) {
          message.success(`成功添加 ${successCount} 个API密钥`);
        } else {
          message.warning(`成功: ${successCount} 个, 失败: ${failCount} 个`);
        }
        
        setIsModalVisible(false);
        form.resetFields();
        fetchApiKeys();
        return;
      }

      // 单个添加模式
      if (platform === 'fofa' && values.email) {
        await invoke('add_api_key', { 
          platform, 
          apiKey: values.apiKey,
          email: values.email 
        });
      } else {
        await invoke('add_api_key', { 
          platform, 
          apiKey: values.apiKey 
        });
      }
      
      message.success('API密钥添加成功');
      setIsModalVisible(false);
      form.resetFields();
      fetchApiKeys();
    } catch (error) {
      console.error('添加API密钥出错:', error);
      message.error(`添加API密钥失败: ${error}`);
    }
  };

  // 删除API密钥
  const deleteApiKey = async (apiKey: string, email?: string) => {
    try {
      if (platform === 'fofa' && email) {
        await invoke('delete_api_key', { 
          platform, 
          apiKey: apiKey,
          email 
        });
      } else {
        await invoke('delete_api_key', { 
          platform, 
          apiKey: apiKey 
        });
      }
      
      message.success('API密钥删除成功');
      fetchApiKeys();
    } catch (error) {
      console.error('删除API密钥出错:', error);
      message.error('删除API密钥失败');
    }
  };

  // 验证API密钥
  const validateApiKey = async (apiKey: string, email?: string) => {
    try {
      let result;
      
      if (platform === 'fofa' && email) {
        result = await invoke('validate_api_key', { 
          platform, 
          apiKey: apiKey,
          email 
        });
      } else {
        result = await invoke('validate_api_key', { 
          platform, 
          apiKey: apiKey 
        });
      }
      
      const data = result as { valid: boolean, message?: string, quota?: string };
      
      // 更新API密钥状态
      setApiKeys(prev => prev.map(item => {
        if (item.key === apiKey) {
          return {
            ...item,
            status: data.valid ? 'valid' : 'invalid',
            quota: data.quota
          };
        }
        return item;
      }));
      
      if (data.valid) {
        message.success(`API密钥有效${data.quota ? `，剩余额度: ${data.quota}` : ''}`);
      } else {
        message.error(`API密钥无效: ${data.message || '验证失败'}`);
      }
    } catch (error) {
      console.error('验证API密钥出错:', error);
      message.error('验证API密钥失败');
    }
  };

  // 批量验证所有API密钥
  const validateAllApiKeys = async () => {
    if (apiKeys.length === 0) {
      message.warning('没有API密钥可验证');
      return;
    }
    
    setLoading(true);
    message.loading('正在验证所有API密钥...', 0);
    
    try {
      for (const item of apiKeys) {
        await validateApiKey(item.key, item.email);
      }
      message.destroy();
      message.success('所有API密钥验证完成');
    } catch (error) {
      console.error('批量验证API密钥出错:', error);
      message.destroy();
      message.error('批量验证失败');
    } finally {
      setLoading(false);
    }
  };

  // 表格列定义
  const getColumns = () => {
    const baseColumns = [
      {
        title: 'API密钥',
        dataIndex: 'key',
        key: 'key',
        render: (text: string) => {
          // 显示掩码版本的API密钥
          const masked = text.length > 8 
            ? `${text.substring(0, 4)}...${text.substring(text.length - 4)}`
            : text;
          return masked;
        }
      },
      {
        title: '状态',
        dataIndex: 'status',
        key: 'status',
        render: (status: string, record: ApiKey) => {
          if (status === 'valid') {
            return (
              <Space>
                <Tag color="success">有效</Tag>
                {record.quota && <span>剩余额度: {record.quota}</span>}
              </Space>
            );
          } else if (status === 'invalid') {
            return <Tag color="error">无效</Tag>;
          } else {
            return <Tag color="default">未验证</Tag>;
          }
        }
      },
      {
        title: '操作',
        key: 'action',
        render: (_: any, record: ApiKey) => (
          <Space size="middle">
            <Button 
              size="small" 
              type="primary"
              icon={<CheckCircleOutlined />}
              onClick={() => validateApiKey(record.key, record.email)}
            >
              验证
            </Button>
            <Popconfirm
              title="确定要删除此API密钥吗？"
              onConfirm={() => deleteApiKey(record.key, record.email)}
              okText="确定"
              cancelText="取消"
              icon={<ExclamationCircleOutlined style={{ color: 'red' }} />}
            >
              <Button 
                size="small" 
                danger
                icon={<DeleteOutlined />}
              >
                删除
              </Button>
            </Popconfirm>
          </Space>
        )
      }
    ];
    
    // FOFA平台需要额外显示邮箱列
    if (platform === 'fofa') {
      return [
        ...baseColumns.slice(0, 1),
        {
          title: '邮箱',
          dataIndex: 'email',
          key: 'email',
        },
        ...baseColumns.slice(1)
      ];
    }
    
    return baseColumns;
  };

  // 创建平台选项卡
  const tabItems = [
    { key: 'hunter', label: 'Hunter' },
    { key: 'fofa', label: 'FOFA' },
    { key: 'quake', label: 'Quake' },
    { key: 'daydaymap', label: 'DayDayMap' }
  ];

  return (
    <Card 
      title="API密钥管理" 
      variant="outlined"
      extra={
        <Space>
          <Button 
            type="primary" 
            icon={<PlusOutlined />}
            onClick={() => setIsModalVisible(true)}
          >
            添加API密钥
          </Button>
          <Button 
            onClick={validateAllApiKeys}
            loading={loading}
          >
            验证全部
          </Button>
        </Space>
      }
    >
      <Tabs activeKey={platform} onChange={setPlatform} items={tabItems} />
      
      <Table 
        columns={getColumns()} 
        dataSource={apiKeys}
        rowKey="key"
        loading={loading}
        pagination={false}
      />
      
      <Modal
        title={
          <Space>
            <span>添加API密钥</span>
            <Button 
              type="link" 
              size="small"
              onClick={() => {
                setIsBatchMode(!isBatchMode);
                form.resetFields();
              }}
            >
              {isBatchMode ? '切换到单个添加' : '切换到批量添加'}
            </Button>
          </Space>
        }
        open={isModalVisible}
        onCancel={() => {
          setIsModalVisible(false);
          setIsBatchMode(false);
          form.resetFields();
        }}
        footer={null}
        width={isBatchMode ? 600 : 520}
      >
        <Form
          form={form}
          layout="vertical"
          onFinish={addApiKey}
        >
          {isBatchMode ? (
            // 批量添加模式
            <>
              <Form.Item
                name="batchKeys"
                label={
                  <Space>
                    <span>API密钥列表</span>
                    <Tag color="blue">每行一个密钥</Tag>
                  </Space>
                }
                rules={[{ required: true, message: '请输入API密钥列表' }]}
              >
                <Input.TextArea 
                  placeholder={`请输入${platform.toUpperCase()} API密钥，每行一个\n例如：\nkey1\nkey2\nkey3`}
                  rows={8}
                  style={{ fontFamily: 'monospace' }}
                />
              </Form.Item>
              
              {platform === 'fofa' && (
                <Form.Item
                  name="email"
                  label="邮箱"
                  rules={[
                    { required: true, message: '请输入FOFA邮箱' },
                    { type: 'email', message: '请输入有效的邮箱地址' }
                  ]}
                  extra="批量添加时，所有密钥将使用同一个邮箱"
                >
                  <Input placeholder="请输入FOFA账号邮箱" />
                </Form.Item>
              )}
            </>
          ) : (
            // 单个添加模式
            <>
              <Form.Item
                name="apiKey"
                label="API密钥"
                rules={[{ required: true, message: '请输入API密钥' }]}
              >
                <Input placeholder={`请输入${platform.toUpperCase()} API密钥`} />
              </Form.Item>
              
              {platform === 'fofa' && (
                <Form.Item
                  name="email"
                  label="邮箱"
                  rules={[
                    { required: true, message: '请输入FOFA邮箱' },
                    { type: 'email', message: '请输入有效的邮箱地址' }
                  ]}
                >
                  <Input placeholder="请输入FOFA账号邮箱" />
                </Form.Item>
              )}
            </>
          )}
          
          <Form.Item>
            <Button type="primary" htmlType="submit" block size="large">
              {isBatchMode ? '批量添加' : '添加'}
            </Button>
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
};

export default ApiKeyManagement; 