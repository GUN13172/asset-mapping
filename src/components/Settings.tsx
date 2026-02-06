import React, { useState, useEffect } from 'react';
import { Card, Form, Input, Button, Select, Switch, Divider, message, Space } from 'antd';
import { SaveOutlined, ReloadOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/tauri';
import { useTheme } from '../hooks/useTheme';

interface Settings {
  exportPath: string;
  defaultPlatform: string;
  pageSize: number;
  autoValidateApiKeys: boolean;
  theme: 'light' | 'dark' | 'system';
  language: 'zh_CN' | 'en_US';
}

const Settings: React.FC = () => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState<boolean>(false);
  const { theme, setTheme } = useTheme();

  // 加载设置
  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    setLoading(true);
    try {
      const settings = await invoke('get_settings') as Settings;
      form.setFieldsValue(settings);
    } catch (error) {
      console.error('加载设置出错:', error);
      message.error('加载设置失败');
    } finally {
      setLoading(false);
    }
  };

  // 保存设置
  const saveSettings = async (values: Settings) => {
    setLoading(true);
    try {
      // 手动添加当前主题到设置中
      const settingsWithTheme = {
        ...values,
        theme: theme
      };
      await invoke('save_settings', { settings: settingsWithTheme });
      message.success('设置保存成功');
    } catch (error) {
      console.error('保存设置出错:', error);
      message.error('保存设置失败');
    } finally {
      setLoading(false);
    }
  };

  // 主题切换处理 - 立即生效
  const handleThemeChange = (newTheme: 'light' | 'dark' | 'system') => {
    console.log('🎨 Settings: 主题切换开始', newTheme);
    console.log('🎨 Settings: 当前主题', theme);
    setTheme(newTheme);
    console.log('🎨 Settings: 主题切换完成');
  };

  // 选择导出路径
  const selectExportPath = async () => {
    try {
      const path = await invoke('select_directory') as string;
      if (path) {
        form.setFieldValue('exportPath', path);
      }
    } catch (error) {
      console.error('选择目录出错:', error);
      message.error('选择目录失败');
    }
  };

  // 重置设置
  const resetSettings = () => {
    form.resetFields();
    message.info('设置已重置');
  };

  // 平台选项
  const platformOptions = [
    { value: 'hunter', label: 'Hunter' },
    { value: 'fofa', label: 'FOFA' },
    { value: 'quake', label: 'Quake' },
    { value: 'daydaymap', label: 'DayDayMap' }
  ];

  // 页码选项
  const pageSizeOptions = [
    { value: 10, label: '10条/页' },
    { value: 20, label: '20条/页' },
    { value: 50, label: '50条/页' },
    { value: 100, label: '100条/页' }
  ];

  // 主题选项
  const themeOptions = [
    { value: 'light', label: '浅色' },
    { value: 'dark', label: '深色' },
    { value: 'system', label: '跟随系统' }
  ];

  // 语言选项
  const languageOptions = [
    { value: 'zh_CN', label: '中文(简体)' },
    { value: 'en_US', label: 'English' }
  ];

  return (
    <Card title="系统设置" variant="outlined">
      <Form
        form={form}
        layout="vertical"
        onFinish={saveSettings}
        initialValues={{
          exportPath: '',
          defaultPlatform: 'hunter',
          pageSize: 20,
          autoValidateApiKeys: true,
          theme: theme,
          language: 'zh_CN'
        }}
      >
        <Card title="基本设置" size="small" variant="outlined">
          <Form.Item
            name="exportPath"
            label="导出路径"
            rules={[{ required: true, message: '请选择导出路径' }]}
          >
            <Space.Compact style={{ width: '100%' }}>
              <Input readOnly />
              <Button onClick={selectExportPath}>选择目录</Button>
            </Space.Compact>
          </Form.Item>

          <Form.Item
            name="defaultPlatform"
            label="默认平台"
            rules={[{ required: true, message: '请选择默认平台' }]}
          >
            <Select options={platformOptions} />
          </Form.Item>

          <Form.Item
            name="pageSize"
            label="默认每页条数"
            rules={[{ required: true, message: '请选择默认每页条数' }]}
          >
            <Select options={pageSizeOptions} />
          </Form.Item>
        </Card>

        <Divider />

        <Card title="API设置" size="small" variant="outlined">
          <Form.Item
            name="autoValidateApiKeys"
            label="自动验证API密钥"
            valuePropName="checked"
          >
            <Switch />
          </Form.Item>
          <p className="text-muted">启用后，添加API密钥时会自动进行验证</p>
        </Card>

        <Divider />

        <Card title="界面设置" size="small" variant="outlined">
          <Form.Item
            label="主题"
          >
            <Select 
              options={themeOptions}
              value={theme}
              onChange={handleThemeChange}
            />
          </Form.Item>

          <Form.Item
            name="language"
            label="语言"
          >
            <Select options={languageOptions} />
          </Form.Item>
        </Card>

        <Divider />

        <Form.Item>
          <Space>
            <Button
              type="primary"
              htmlType="submit"
              icon={<SaveOutlined />}
              loading={loading}
            >
              保存设置
            </Button>
            <Button
              icon={<ReloadOutlined />}
              onClick={resetSettings}
            >
              重置设置
            </Button>
          </Space>
        </Form.Item>
      </Form>
    </Card>
  );
};

export default Settings; 