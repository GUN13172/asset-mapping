import React, { useState, useCallback } from 'react';
import { Card, Tabs, Input, Button, Table, Select, Space, Checkbox, AutoComplete, message, Tag } from 'antd';
import { SearchOutlined, DownloadOutlined, SendOutlined, BugOutlined, ThunderboltOutlined } from '@ant-design/icons';
import { invoke } from '@tauri-apps/api/core';
import ProgressModal, { ProgressStatus, ProgressLog } from './ProgressModal';
import { fingerprints, Fingerprint } from '../data/fingerprints';
import { useExportProgress } from '../hooks/useExportProgress';
import { endPerf, startPerf } from '../utils/perf';
import { normalizeSmartPunctuation } from '../utils/textInput';

interface AssetResult {
  url: string;
  ip: string;
  port: string;
  web_title?: string;
  country?: string;
  province?: string;
  city?: string;
  server?: string;
  source?: string;
}

const ALL_PLATFORMS = ['hunter', 'fofa', 'quake', 'daydaymap'] as const;
const EMPTY_PLATFORM_TOTALS: Record<string, number> = {
  hunter: 0,
  fofa: 0,
  quake: 0,
  daydaymap: 0,
};

const AssetQuery: React.FC = () => {
  const [platform, setPlatform] = useState<string>('hunter');
  const [query, setQuery] = useState<string>('');
  const queryRef = React.useRef(query);
  const [appendToQuery, setAppendToQuery] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);
  const [results, setResults] = useState<AssetResult[]>([]);
  const [totalResults, setTotalResults] = useState<number>(0);
  const [platformTotals, setPlatformTotals] = useState<Record<string, number>>(EMPTY_PLATFORM_TOTALS);
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [pageSize, setPageSize] = useState<number>(20);
  const [province, setProvince] = useState<string>('');
  const [city, setCity] = useState<string>('');

  // 中国省市联动数据
  const provinceCityMap: Record<string, string[]> = {
    '北京市': ['东城区', '西城区', '朝阳区', '海淀区', '丰台区', '通州区', '大兴区', '顺义区', '昌平区'],
    '上海市': ['浦东新区', '黄浦区', '徐汇区', '静安区', '长宁区', '虹口区', '杨浦区', '闵行区', '宝山区', '嘉定区', '松江区'],
    '天津市': ['和平区', '河东区', '河西区', '南开区', '河北区', '红桥区', '滨海新区', '武清区'],
    '重庆市': ['渝中区', '江北区', '南岸区', '沙坪坝区', '九龙坡区', '渝北区', '巴南区'],
    '广东省': ['广州市', '深圳市', '东莞市', '佛山市', '珠海市', '中山市', '惠州市', '汕头市', '江门市', '湛江市', '肇庆市', '茂名市'],
    '浙江省': ['杭州市', '宁波市', '温州市', '嘉兴市', '湖州市', '绍兴市', '金华市', '台州市', '丽水市'],
    '江苏省': ['南京市', '苏州市', '无锡市', '常州市', '南通市', '徐州市', '盐城市', '扬州市', '镇江市', '泰州市', '淮安市', '连云港市'],
    '山东省': ['济南市', '青岛市', '烟台市', '威海市', '潍坊市', '淄博市', '临沂市', '济宁市', '泰安市', '日照市', '德州市'],
    '河南省': ['郑州市', '洛阳市', '开封市', '南阳市', '新乡市', '许昌市', '平顶山市', '安阳市', '焦作市'],
    '四川省': ['成都市', '绵阳市', '德阳市', '宜宾市', '泸州市', '达州市', '南充市', '自贡市', '乐山市', '内江市'],
    '湖北省': ['武汉市', '宜昌市', '襄阳市', '荆州市', '十堰市', '黄冈市', '孝感市', '荆门市'],
    '湖南省': ['长沙市', '株洲市', '湘潭市', '衡阳市', '邵阳市', '岳阳市', '常德市', '张家界市', '郴州市'],
    '福建省': ['福州市', '厦门市', '泉州市', '漳州市', '莆田市', '宁德市', '龙岩市', '三明市', '南平市'],
    '安徽省': ['合肥市', '芜湖市', '蚌埠市', '淮南市', '马鞍山市', '安庆市', '黄山市', '滁州市', '阜阳市', '宿州市'],
    '河北省': ['石家庄市', '唐山市', '秦皇岛市', '邯郸市', '保定市', '张家口市', '廊坊市', '沧州市', '承德市', '衡水市'],
    '陕西省': ['西安市', '咸阳市', '宝鸡市', '渭南市', '汉中市', '安康市', '延安市', '榆林市', '商洛市'],
    '山西省': ['太原市', '大同市', '运城市', '临汾市', '晋中市', '长治市', '晋城市', '阳泉市', '朔州市'],
    '辽宁省': ['沈阳市', '大连市', '鞍山市', '抚顺市', '本溪市', '丹东市', '锦州市', '营口市', '阜新市', '盘锦市'],
    '吉林省': ['长春市', '吉林市', '四平市', '通化市', '白山市', '松原市', '白城市'],
    '黑龙江省': ['哈尔滨市', '齐齐哈尔市', '牡丹江市', '佳木斯市', '大庆市', '鸡西市', '双鸭山市', '伊春市', '绥化市'],
    '江西省': ['南昌市', '九江市', '景德镇市', '赣州市', '吉安市', '宜春市', '抚州市', '上饶市', '萍乡市', '新余市'],
    '云南省': ['昆明市', '大理市', '丽江市', '曲靖市', '玉溪市', '保山市', '昭通市', '普洱市', '临沧市'],
    '贵州省': ['贵阳市', '遵义市', '安顺市', '毕节市', '铜仁市', '六盘水市'],
    '甘肃省': ['兰州市', '天水市', '白银市', '平凉市', '酒泉市', '庆阳市', '定西市', '陇南市', '张掖市', '武威市'],
    '广西壮族自治区': ['南宁市', '柳州市', '桂林市', '梧州市', '北海市', '防城港市', '钦州市', '贵港市', '玉林市'],
    '海南省': ['海口市', '三亚市', '儋州市', '琼海市', '万宁市', '文昌市'],
    '内蒙古自治区': ['呼和浩特市', '包头市', '赤峰市', '鄂尔多斯市', '通辽市', '呼伦贝尔市', '乌海市', '乌兰察布市'],
    '新疆维吾尔自治区': ['乌鲁木齐市', '克拉玛依市', '喀什市', '吐鲁番市', '哈密市', '阿克苏市', '库尔勒市'],
    '西藏自治区': ['拉萨市', '日喀则市', '昌都市', '林芝市', '山南市', '那曲市'],
    '宁夏回族自治区': ['银川市', '石嘴山市', '吴忠市', '固原市', '中卫市'],
    '青海省': ['西宁市', '海东市', '格尔木市', '德令哈市'],
    '香港特别行政区': ['中西区', '湾仔区', '东区', '南区', '油尖旺区', '深水埗区', '九龙城区'],
    '澳门特别行政区': ['澳门半岛', '氹仔', '路环'],
    '台湾省': ['台北市', '高雄市', '台中市', '台南市', '新竹市', '基隆市'],
  };

  // 中国省市联动数据
  const [autoCompleteOptions, setAutoCompleteOptions] = useState<{ value: string; label: React.ReactNode }[]>([]);
  const [aggregatedSearch, setAggregatedSearch] = useState<boolean>(false);
  const [convertedQueries, setConvertedQueries] = useState<Record<string, string>>({});

  // 搜索进度弹窗状态
  const [searchModalOpen, setSearchModalOpen] = useState(false);
  const [searchStatus, setSearchStatus] = useState<ProgressStatus>('idle');
  const [searchPercent, setSearchPercent] = useState(0);
  const [searchStatusText, setSearchStatusText] = useState('');
  const [searchLogs, setSearchLogs] = useState<ProgressLog[]>([]);

  // 导出进度 hook
  const exportProgress = useExportProgress();

  // 平台查询语法提示
  const syntaxHelp = {
    hunter: [
      { label: 'domain.suffix="test.com"', description: '搜索域名后缀' },
      { label: 'ip="1.1.1.1"', description: '搜索IP' },
      { label: 'web.title="登录"', description: '搜索网页标题' },
      { label: 'header="thinkphp"', description: '搜索HTTP头' },
      { label: 'app.name="ThinkPHP"', description: '搜索应用框架' },
      { label: 'port="3306"', description: '搜索端口' },
      { label: 'status_code="200"', description: '搜索状态码' },
      { label: 'protocol="https"', description: '搜索协议' },
      { label: 'ip.province="北京市"', description: '搜索省份' },
      { label: 'ip.city="北京市"', description: '搜索城市' },
      { label: 'ip.country="中国"', description: '搜索国家' },
      { label: 'web.body="login"', description: '搜索网页内容' },
      { label: 'cert="baidu"', description: '搜索证书' },
      { label: 'banner="nginx"', description: '搜索Banner' },
    ],
    fofa: [
      { label: 'domain="test.com"', description: '搜索域名' },
      { label: 'ip="1.1.1.1"', description: '搜索IP' },
      { label: 'title="登录"', description: '搜索网页标题' },
      { label: 'header="nginx"', description: '搜索HTTP头' },
      { label: 'server=="Microsoft-IIS/10"', description: '搜索服务器' },
      { label: 'port="6379"', description: '搜索端口' },
      { label: 'protocol="https"', description: '搜索协议' },
      { label: 'country="CN"', description: '搜索国家' },
      { label: 'region="Beijing"', description: '搜索地区' },
      { label: 'city="Beijing"', description: '搜索城市' },
      { label: 'body="login"', description: '搜索网页内容' },
      { label: 'cert="baidu"', description: '搜索证书' },
      { label: 'banner="nginx"', description: '搜索Banner' },
    ],
    quake: [
      { label: 'domain: test.com', description: '搜索域名' },
      { label: 'ip: "1.1.1.1"', description: '搜索IP' },
      { label: 'title: "登录"', description: '搜索网页标题' },
      { label: 'response: "nginx"', description: '搜索响应内容' },
      { label: 'service: "IIS"', description: '搜索服务' },
      { label: 'port: 3389', description: '搜索端口' },
      { label: 'protocol: "https"', description: '搜索协议' },
      { label: 'country: "China"', description: '搜索国家' },
      { label: 'province: "Beijing"', description: '搜索省份' },
      { label: 'city: "Beijing"', description: '搜索城市' },
      { label: 'cert: "baidu"', description: '搜索证书' },
      { label: 'banner: "nginx"', description: '搜索Banner' },
    ],
    daydaymap: [
      { label: 'domain:"test.com"', description: '搜索域名' },
      { label: 'ip:"1.1.1.1"', description: '搜索IP地址' },
      { label: 'ip:"1.1.1.0/24"', description: '搜索IP段（CIDR）' },
      { label: 'title:"登录"', description: '搜索网页标题' },
      { label: 'server:"nginx"', description: '搜索服务器' },
      { label: 'app:"WordPress"', description: '搜索应用' },
      { label: 'port:"80"', description: '搜索端口' },
      { label: 'protocol:"https"', description: '搜索协议' },
      { label: 'country:"中国"', description: '搜索国家' },
      { label: 'province:"北京"', description: '搜索省份' },
      { label: 'city:"北京"', description: '搜索城市' },
      { label: 'body:"login"', description: '搜索网页内容' },
      { label: 'cert:"baidu"', description: '搜索证书' },
      { label: 'banner:"nginx"', description: '搜索Banner' },
    ],
  };

  // 查询占位符
  const placeholders = {
    hunter: '例如: domain.suffix="test.com" && ip.province="北京市"',
    fofa: '例如: domain="test.com" && country="CN"',
    quake: '例如: domain: test.com AND country: "China"',
    daydaymap: '例如: ip:"1.1.1.0/24" 或 domain:"test.com" (注意：使用冒号和引号)',
  };

  const platformMeta = {
    hunter: {
      label: 'Hunter',
      description: '适合做高覆盖面资产检索与快速资产摸排。',
      accent: '#fa8c16',
    },
    fofa: {
      label: 'FOFA',
      description: '偏向语法灵活与指纹场景，适合做特征定位。',
      accent: '#1677ff',
    },
    quake: {
      label: 'Quake',
      description: '更适合结构化条件检索与服务画像分析。',
      accent: '#722ed1',
    },
    daydaymap: {
      label: 'DayDayMap',
      description: '对地理与资产分布信息更直观，适合快速筛查。',
      accent: '#13c2c2',
    },
  };
  const currentPlatformMeta = platformMeta[platform as keyof typeof platformMeta];
  const modeLabel = aggregatedSearch ? '全平台聚合' : `${currentPlatformMeta.label} 单平台`;

  // 表格列定义
  const columns = [
    {
      title: '来源',
      dataIndex: 'source',
      key: 'source',
      width: 140,
      render: (platform: string) => {
        const colors: Record<string, string> = {
          fofa: '#1677ff',
          hunter: '#fa8c16',
          quake: '#722ed1',
          daydaymap: '#13c2c2'
        };
        return <Tag color={colors[platform] || 'blue'}>{platform.toUpperCase()}</Tag>;
      }
    },
    {
      title: 'URL',
      dataIndex: 'url',
      key: 'url',
      width: 280,
      ellipsis: true,
      render: (text: string) => {
        if (!text) return '-';
        // 确保没有 http 前缀时自动加上，避免相对路径跳转
        let href = text;
        if (!href.match(/^[a-zA-Z]+:\/\//)) {
          href = `http://${href}`;
        }
        return <a href={href} target="_blank" rel="noopener noreferrer">{text}</a>;
      },
    },
    {
      title: 'IP',
      dataIndex: 'ip',
      key: 'ip',
      width: 150,
    },
    {
      title: '端口',
      dataIndex: 'port',
      key: 'port',
      width: 90,
    },
    {
      title: '标题',
      dataIndex: 'web_title',
      key: 'web_title',
      width: 220,
      ellipsis: true,
    },
    {
      title: '国家',
      dataIndex: 'country',
      key: 'country',
      width: 90,
    },
    {
      title: '省份',
      dataIndex: 'province',
      key: 'province',
      width: 100,
    },
    {
      title: '城市',
      dataIndex: 'city',
      key: 'city',
      width: 100,
    },
    {
      title: '服务器',
      dataIndex: 'server',
      key: 'server',
      width: 180,
      ellipsis: true,
    },
    {
      title: '操作',
      key: 'action',
      width: 180,
      fixed: 'right' as const,
      className: 'asset-query-action-cell',
      render: (_: any, record: AssetResult) => (
        <Space size="small" className="asset-query-actions" wrap={false}>
          <Button
            type="link"
            size="small"
            icon={<SendOutlined />}
            onClick={() => handleSendToResender(record)}
          >
            重发
          </Button>
          <Button
            type="link"
            size="small"
            icon={<BugOutlined />}
            style={{ color: '#ff4d4f' }}
            onClick={() => handleSendToScanner(record)}
          >
            扫描
          </Button>
        </Space>
      ),
    },
  ];

  // 发送到扫描器
  const handleSendToScanner = (record: AssetResult) => {
    const target = record.url || record.ip;
    if (target) {
      localStorage.setItem('pending_scan_target', target);
      message.success(`目标 ${target} 已发送到漏洞扫描模块`);
    }
  };

  // 发送到重发器
  const handleSendToResender = (record: AssetResult) => {
    const target = record.url || record.ip;
    if (!target) return;
    try {
      const host = new URL(target.startsWith('http') ? target : `http://${target}`).host;
      const rawRequest = `GET / HTTP/1.1\nHost: ${host}\nUser-Agent: Mozilla/5.0\nAccept: */*\n\n`;
      localStorage.setItem('pending_resend_request', rawRequest);
      message.success(`资产 ${target} 已发送到重发器，请切换到重发器页面查看`);
    } catch {
      message.error('无法解析目标地址');
    }
  };

  // 处理平台切换
  const handlePlatformChange = (value: string) => {
    setPlatform(value);
    setQuery('');
    queryRef.current = '';
    setResults([]);
    setAutoCompleteOptions([]);
    setConvertedQueries({});
  };

  // 实时转换逻辑
  const handleRealtimeConvert = async (q: string) => {
    try {
      const results = await invoke<{ platform: string; query: string }[]>('convert_query_to_all', {
        query: q,
        fromPlatform: platform
      });
      const mapped: Record<string, string> = {};
      results.forEach(r => mapped[r.platform] = r.query);
      setConvertedQueries(mapped);
    } catch (e) {
      // 忽略转换失败
    }
  };

  // 处理输入变化，生成联想提示
  const handleQueryChange = (value: string) => {
    const normalizedValue = normalizeSmartPunctuation(value);
    setQuery(normalizedValue);
    queryRef.current = normalizedValue;

    // 获取当前平台的语法提示
    const currentSyntax = syntaxHelp[platform as keyof typeof syntaxHelp];

    // 如果输入为空，显示所有提示
    if (!normalizedValue.trim()) {
      const options = currentSyntax.map(item => ({
        value: item.label,
        label: (
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 500, color: 'var(--accent-cyan)' }}>{item.label}</span>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{item.description}</span>
          </div>
        ),
      }));
      setAutoCompleteOptions(options);
      setConvertedQueries({});
      return;
    }

    // 获取最后一个词（用于智能匹配）
    const lastWord = normalizedValue.split(/[\s&|()]/).pop()?.toLowerCase() || '';

    // 生成联想选项
    const fpOptions = fingerprints
      .filter(f => f.name.toLowerCase().includes(lastWord) || f.category.toLowerCase().includes(lastWord))
      .map(f => ({
        value: f[platform as keyof Pick<Fingerprint, 'fofa' | 'hunter' | 'quake'>] || f.fofa,
        label: (
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 'bold', color: 'var(--accent-green)' }}>{f.name}</span>
            <span style={{ fontSize: '11px', color: 'var(--text-muted)' }}>{f.category}</span>
          </div>
        ),
      }));

    const stOptions = currentSyntax
      .filter(item => {
        const label = item.label.toLowerCase();
        const desc = item.description.toLowerCase();
        return label.includes(lastWord) || desc.includes(lastWord);
      })
      .map(item => ({
        value: item.label,
        label: (
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 500, color: 'var(--accent-cyan)' }}>{item.label}</span>
            <span style={{ fontSize: '12px', color: 'var(--text-muted)' }}>{item.description}</span>
          </div>
        ),
      }));

    setAutoCompleteOptions([...fpOptions, ...stOptions]);

    // 实时转换预览
    handleRealtimeConvert(normalizedValue);
  };

  // 处理选择联想项
  const handleSelect = (value: string) => {
    const normalizedValue = normalizeSmartPunctuation(value);

    // 如果当前查询为空，直接设置
    if (!query.trim()) {
      setQuery(normalizedValue);
      queryRef.current = normalizedValue;
      handleRealtimeConvert(normalizedValue);
      handleSearch(1, pageSize, normalizedValue);
      return;
    }

    // 否则追加到当前查询
    const connector = platform === 'quake' || platform === 'daydaymap' ? ' AND ' : ' && ';
    const newQuery = query + connector + normalizedValue;
    setQuery(newQuery);
    queryRef.current = newQuery;
    handleRealtimeConvert(newQuery);
    handleSearch(1, pageSize, newQuery);
  };

  // 处理搜索（带进度弹窗）
  const handleSearch = useCallback(async (searchPage?: number, searchPageSize?: number, customQuery?: string) => {
    const rawQuery = customQuery ?? queryRef.current;
    const activeQuery = normalizeSmartPunctuation(rawQuery);
    if (!activeQuery.trim()) {
      return;
    }
    if (activeQuery !== rawQuery) {
      setQuery(activeQuery);
      queryRef.current = activeQuery;
    }

    const page = searchPage ?? currentPage;
    const size = searchPageSize ?? pageSize;

    // 清空上次查询结果（如果是新查询）
    if (page === 1) {
      setResults([]);
      setTotalResults(0);
    }

    // 显示搜索进度弹窗
    setSearchModalOpen(true);
    setSearchStatus('running');
    setSearchPercent(30);
    setSearchStatusText(`正在查询 [${platform.toUpperCase()}] 第 ${page} 页...`);
    setSearchLogs([{
      time: new Date().toLocaleTimeString(),
      message: `发起查询: 平台=${platform}, 页码=${page}, 每页=${size}`,
      type: 'info',
    }]);

    setLoading(true);
    const searchPerfToken = startPerf('asset-search', {
      platform,
      page,
      pageSize: size,
      aggregated: aggregatedSearch,
    });
    try {
      setSearchPercent(60);

      let finalResults: any[] = [];
      let finalTotal = 0;
      const nextPlatformTotals = { ...EMPTY_PLATFORM_TOTALS };

      if (aggregatedSearch) {
        // 并发搜索多平台
        setSearchStatusText(`正在聚合查询准备...转换查询语句`);
        let currentQueries = { ...convertedQueries };
        // 若没有转换结果，先主动转换一次
        if (Object.keys(currentQueries).length === 0) {
          try {
            const converts = await invoke<{ platform: string; query: string }[]>('convert_query_to_all', {
              query: activeQuery,
              fromPlatform: platform
            });
            converts.forEach(r => currentQueries[r.platform] = r.query);
            setConvertedQueries(currentQueries);
          } catch (e) {
            console.error('转换查询语句失败:', e);
          }
        }

        setSearchStatusText(`正在并发聚合查询各平台数据...`);
        const promises = ALL_PLATFORMS.map(async (p) => {
          try {
            const q = p === platform ? activeQuery : (currentQueries[p] || activeQuery);
            const res = await invoke<any>('search_assets', {
              platform: p,
              query: q,
              page,
              pageSize: size,
            });
            return { platform: p, data: res };
          } catch (e) {
            console.error(`平台 ${p} 搜索失败:`, e);
            return { platform: p, data: { results: [], total: 0 } };
          }
        });

        const responses = await Promise.all(promises);
        responses.forEach(r => {
          const resWithSource = (r.data.results || []).map((item: any) => ({ ...item, source: r.platform }));
          finalResults = [...finalResults, ...resWithSource];
          const platformTotal = Number(r.data.total || 0);
          nextPlatformTotals[r.platform] = platformTotal;
          finalTotal += platformTotal;
        });
      } else {
        const result = await invoke('search_assets', {
          platform,
          query: activeQuery,
          page,
          pageSize: size,
        });
        const data = result as { total: number; results: any[] };
        finalResults = (data.results || []).map(item => ({ ...item, source: platform }));
        finalTotal = data.total || 0;
        nextPlatformTotals[platform] = finalTotal;
      }

      setResults(finalResults);
      setTotalResults(finalTotal);
      setPlatformTotals(nextPlatformTotals);

      setSearchPercent(100);
      setSearchStatus('success');
      setSearchStatusText(`查询完成！共找到 ${finalTotal} 条结果，本页 ${finalResults.length} 条`);
      setSearchLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✓ 查询完成: 共 ${finalTotal} 条结果`,
        type: 'success',
      }]);
      endPerf(searchPerfToken, {
        platform,
        page,
        pageSize: size,
        aggregated: aggregatedSearch,
        status: 'success',
        totalResults: finalTotal,
        pageResults: finalResults.length,
      });
    } catch (error: any) {
      console.error('查询出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`查询出错: ${errMsg}`);

      setResults([]);
      setTotalResults(0);
      setPlatformTotals(EMPTY_PLATFORM_TOTALS);

      setSearchPercent(100);
      setSearchStatus('error');
      setSearchStatusText(`查询失败: ${errMsg}`);
      setSearchLogs(prev => [...prev, {
        time: new Date().toLocaleTimeString(),
        message: `✗ 查询失败: ${errMsg}`,
        type: 'error',
      }]);
      endPerf(searchPerfToken, {
        platform,
        page,
        pageSize: size,
        aggregated: aggregatedSearch,
        status: 'error',
        error: errMsg,
      });
    } finally {
      setLoading(false);
    }
  }, [query, platform, currentPage, pageSize, aggregatedSearch, convertedQueries]);

  // 处理页码变化 - 切换页码后自动触发搜索
  const handlePageChange = (page: number, newPageSize?: number) => {
    setCurrentPage(page);
    if (newPageSize) setPageSize(newPageSize);
    handleSearch(page, newPageSize ?? pageSize);
  };

  // 应用地理位置筛选
  const applyLocationFilter = () => {
    let locationQuery = '';

    if (province) {
      switch (platform) {
        case 'hunter': locationQuery += `ip.province="${province}"`; break;
        case 'fofa': locationQuery += `region="${province}"`; break;
        case 'quake': locationQuery += `province: "${province}"`; break;
        case 'daydaymap': locationQuery += `province:"${province}"`; break;
      }
    }

    if (city) {
      if (locationQuery) {
        locationQuery += (platform === 'quake' || platform === 'daydaymap') ? ' AND ' : ' && ';
      }
      switch (platform) {
        case 'hunter': locationQuery += `ip.city="${city}"`; break;
        case 'fofa': locationQuery += `city="${city}"`; break;
        case 'quake': locationQuery += `city: "${city}"`; break;
        case 'daydaymap': locationQuery += `city:"${city}"`; break;
      }
    }

    if (locationQuery) {
      let newQuery = '';
      if (appendToQuery && queryRef.current) {
        const connector = (platform === 'quake' || platform === 'daydaymap') ? ' AND ' : ' && ';
        newQuery = `${queryRef.current}${connector}${locationQuery}`;
      } else {
        newQuery = locationQuery;
      }
      setQuery(newQuery);
      queryRef.current = newQuery;
      handleRealtimeConvert(newQuery);
      handleSearch(1, pageSize, newQuery);
    }
  };

  // 导出结果
  const exportResults = async () => {
    if (results.length === 0) {
      message.warning('没有可导出的结果');
      return;
    }

    const normalizedQuery = normalizeSmartPunctuation(query).trim();
    if (!normalizedQuery) {
      message.warning('查询语句不能为空');
      return;
    }

    const computePages = (total: number) => Math.max(1, Math.ceil(total / pageSize));
    const totalPages = aggregatedSearch
      ? ALL_PLATFORMS.reduce((sum, targetPlatform) => {
        const platformTotal = platformTotals[targetPlatform] || 0;
        return sum + (platformTotal > 0 ? computePages(platformTotal) : 0);
      }, 0)
      : computePages(totalResults);

    const taskId = `export_${Date.now()}`;
    exportProgress.startTask(
      taskId,
      aggregatedSearch
        ? `开始导出聚合查询结果: 查询=${normalizedQuery}, 预计总页数=${totalPages}`
        : `开始导出: 平台=${platform}, 查询=${normalizedQuery}, 页数=${totalPages}`
    );
    const exportPerfToken = startPerf('asset-export', {
      source: 'asset-query',
      platform: aggregatedSearch ? 'aggregated' : platform,
      pageSize,
      pages: totalPages,
    });

    try {
      if (aggregatedSearch) {
        let currentQueries = { ...convertedQueries };
        const missingConversions = ALL_PLATFORMS.some(
          (targetPlatform) => targetPlatform !== platform && !currentQueries[targetPlatform]
        );

        if (missingConversions) {
          try {
            const converts = await invoke<{ platform: string; query: string }[]>('convert_query_to_all', {
              query: normalizedQuery,
              fromPlatform: platform
            });
            converts.forEach(r => currentQueries[r.platform] = r.query);
            setConvertedQueries(currentQueries);
          } catch (e) {
            exportProgress.addLog(`⚠ 查询语法转换失败，将尽量使用原始语句继续导出: ${e}`, 'warning');
          }
        }

        let successCount = 0;

        for (const targetPlatform of ALL_PLATFORMS) {
          const platformTotal = platformTotals[targetPlatform] || 0;
          if (platformTotal <= 0) {
            exportProgress.addLog(`- ${targetPlatform} 平台无匹配结果，已跳过`, 'info');
            continue;
          }

          const targetPages = computePages(platformTotal);
          const targetQuery = targetPlatform === platform
            ? normalizedQuery
            : (currentQueries[targetPlatform] || normalizedQuery);

          exportProgress.setStatus('running');
          exportProgress.setStatusText(`正在导出 ${targetPlatform} 平台...`);

          try {
            const filePath = await invoke<string>('export_results_with_progress', {
              taskId: `${taskId}_${targetPlatform}`,
              platform: targetPlatform,
              query: targetQuery,
              pages: targetPages,
              pageSize,
              timeRange: 'all',
            });
            exportProgress.addLog(
              `✓ ${targetPlatform} 导出成功: ${filePath}（共 ${platformTotal} 条，${targetPages} 页）`,
              'success'
            );
            successCount += 1;
          } catch (error: any) {
            const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
            exportProgress.addLog(`✗ ${targetPlatform} 导出失败: ${errMsg}`, 'error');
          }
        }

        if (successCount === 0) {
          throw new Error('没有成功导出任何平台结果');
        }

        exportProgress.setPercent(100);
        exportProgress.setStatus('success');
        exportProgress.setStatusText(`聚合查询导出完成！共导出 ${successCount} 个平台`);
        exportProgress.addLog(`✓ 聚合查询导出完成，共导出 ${successCount} 个平台`, 'success');
        exportProgress.finishTask();
        message.success('导出成功，结果请查看导出目录');
      } else {
        const filePath = await invoke<string>('export_results_with_progress', {
          taskId,
          platform,
          query: normalizedQuery,
          pages: totalPages,
          pageSize,
          timeRange: 'all',
        });
        message.success(`导出成功: ${filePath}`);
      }

      endPerf(exportPerfToken, {
        source: 'asset-query',
        platform: aggregatedSearch ? 'aggregated' : platform,
        status: 'success',
      });
    } catch (error: any) {
      console.error('导出出错:', error);
      const errMsg = typeof error === 'string' ? error : (error?.message || '未知错误');
      message.error(`导出出错: ${errMsg}`);
      exportProgress.setStatus('error');
      exportProgress.setStatusText(`导出失败: ${errMsg}`);
      exportProgress.addLog(`✗ 导出失败: ${errMsg}`, 'error');
      exportProgress.finishTask();
      endPerf(exportPerfToken, {
        source: 'asset-query',
        platform: aggregatedSearch ? 'aggregated' : platform,
        status: 'error',
        error: errMsg,
      });
    }
  };

  // 创建平台选项卡
  const tabItems = [
    { key: 'hunter', label: 'Hunter' },
    { key: 'fofa', label: 'FOFA' },
    { key: 'quake', label: 'Quake' },
    { key: 'daydaymap', label: 'DayDayMap' }
  ];

  // 创建页码选项
  const pageSizeOptions = [10, 20, 50, 100];

  return (
    <Card
      title={
        <div className="asset-query-card-title">
          <div>
            <div className="asset-query-card-eyebrow">Recon Workspace</div>
            <div className="asset-query-card-heading">多平台资产查询</div>
          </div>
          <Tag
            className="asset-query-platform-badge"
            style={{
              color: currentPlatformMeta.accent,
              borderColor: `${currentPlatformMeta.accent}33`,
              background: `${currentPlatformMeta.accent}14`,
            }}
          >
            {modeLabel}
          </Tag>
        </div>
      }
      className="asset-query-page-card glass-effect"
      bordered={false}
    >
      <div className="asset-query-overview">
        <div>
          <div className="asset-query-overview-title">{currentPlatformMeta.label} 检索面板</div>
          <div className="asset-query-overview-text">
            {aggregatedSearch
              ? '当前会自动适配多平台语法并聚合结果，适合做统一视角的资产盘点。'
              : currentPlatformMeta.description}
          </div>
        </div>
        <div className="asset-query-overview-metrics">
          <div className="asset-query-overview-metric">
            <span>当前页</span>
            <strong>{currentPage}</strong>
          </div>
          <div className="asset-query-overview-metric">
            <span>分页容量</span>
            <strong>{pageSize} 条</strong>
          </div>
        </div>
      </div>

      <Tabs
        activeKey={platform}
        onChange={handlePlatformChange}
        items={tabItems}
        className="platform-tabs"
      />

      <div className="query-input-area asset-query-builder">
        <AutoComplete
          style={{ width: '100%', marginBottom: 16 }}
          options={autoCompleteOptions}
          onSelect={handleSelect}
          onChange={handleQueryChange}
          value={query}
          placeholder={placeholders[platform as keyof typeof placeholders]}
          filterOption={false}
          popupClassName="glass-effect"
        >
          <Input.TextArea
            rows={3}
            placeholder={placeholders[platform as keyof typeof placeholders]}
            className="glass-effect"
            autoCorrect="off"
            autoCapitalize="off"
            autoComplete="off"
            spellCheck={false}
            onPressEnter={(e) => {
              if (e.shiftKey) return; // Shift+Enter 换行
              e.preventDefault();
              handleSearch(1, pageSize, normalizeSmartPunctuation((e.target as HTMLTextAreaElement).value));
            }}
          />
        </AutoComplete>

        {Object.keys(convertedQueries).length > 0 && query && (
          <div className="syntax-preview fade-in">
            <Space wrap>
              <span className="syntax-preview-label">实时转换:</span>
              {Object.entries(convertedQueries).map(([p, q]) => (
                <div key={p} className="syntax-preview-chip">
                  <span className="syntax-preview-platform">{p.toUpperCase()}:</span>
                  <span className="syntax-preview-query">{q}</span>
                </div>
              ))}
            </Space>
          </div>
        )}

        <div className="asset-query-toolbar">
          <div className="asset-query-filter-group">
            <AutoComplete
              placeholder="选择或输入省份"
              style={{ width: 140 }}
              value={province}
              onChange={(val) => { setProvince(val); if (!val) setCity(''); }}
              options={Object.keys(provinceCityMap).map(p => ({ value: p }))}
              filterOption={(inputValue, option) =>
                option!.value.toUpperCase().indexOf(inputValue.toUpperCase()) !== -1
              }
              className="glass-effect"
              allowClear
            />

            <AutoComplete
              placeholder="选择或输入城市"
              style={{ width: 140 }}
              value={city}
              onChange={(val) => setCity(val)}
              options={province && provinceCityMap[province] ? provinceCityMap[province].map(c => ({ value: c })) : []}
              filterOption={(inputValue, option) =>
                option!.value.toUpperCase().indexOf(inputValue.toUpperCase()) !== -1
              }
              className="glass-effect"
              allowClear
            />

            <label className="asset-query-check">
              <Checkbox checked={appendToQuery} onChange={(e) => setAppendToQuery(e.target.checked)} />
              <span>追加到当前查询</span>
            </label>

            <Button type="default" onClick={applyLocationFilter} className="glass-effect">
              应用地理筛选
            </Button>
          </div>

          <div className="asset-query-action-group">
            <label className="asset-query-check asset-query-aggregate-check">
              <Checkbox checked={aggregatedSearch} onChange={(e) => setAggregatedSearch(e.target.checked)} />
              <span>
                <ThunderboltOutlined style={{ color: '#faad14', marginRight: 4 }} />
                全平台聚合搜索
              </span>
            </label>

            <Select
              value={pageSize}
              onChange={(value) => setPageSize(value)}
              style={{ width: 120 }}
              className="glass-effect"
            >
              {pageSizeOptions.map(size => (
                <Select.Option key={size} value={size}>{size}条/页</Select.Option>
              ))}
            </Select>

            <Button
              type="primary"
              icon={<SearchOutlined />}
              onClick={() => handleSearch(1, pageSize, query)}
              loading={loading}
              className="search-button gradient-button"
            >
              开始查询
            </Button>
          </div>
        </div>
      </div>

      {totalResults > 0 && (
        <div className="asset-query-summary">
          <div className="asset-query-summary-main">
            <span className="asset-query-summary-label">共找到</span>
            <span className="stats-number">{totalResults}</span>
            <span className="asset-query-summary-label">个资产</span>
          </div>
          <div className="asset-query-summary-grid">
            <div className="asset-query-summary-item">
              <span>查询模式</span>
              <strong>{modeLabel}</strong>
            </div>
            <div className="asset-query-summary-item">
              <span>当前页结果</span>
              <strong>{results.length} 条</strong>
            </div>
            <div className="asset-query-summary-item">
              <span>分页规格</span>
              <strong>{pageSize} 条/页</strong>
            </div>
          </div>
        </div>
      )}

      <div className="asset-query-results-header">
        <div>
          <div className="asset-query-results-eyebrow">Result View</div>
          <div className="asset-query-results-title">资产结果列表</div>
        </div>
        {results.length > 0 && (
          <Button
            type="primary"
            icon={<DownloadOutlined />}
            onClick={exportResults}
            className="gradient-button"
          >
            导出结果
          </Button>
        )}
      </div>

      <div className="results-table glass-effect" style={{ padding: 16, borderRadius: 8 }}>
        <Table
          className="asset-query-table"
          columns={columns}
          dataSource={results}
          rowKey={(record) => record.ip + record.port + record.source}
          loading={loading}
          size="middle"
          tableLayout="fixed"
          scroll={{ x: 1530 }}
          pagination={{
            current: currentPage,
            pageSize,
            total: totalResults,
            onChange: handlePageChange,
            showSizeChanger: true,
            pageSizeOptions: pageSizeOptions.map(size => size.toString()),
            showTotal: (total, range) => `第 ${range[0]}-${range[1]} 条，共 ${total} 条`,
          }}
        />
      </div>

      <Card title="语法提示" size="small" style={{ marginTop: 24 }} className="glass-effect" bordered={false}>
        <div className="syntax-help-tags">
          {syntaxHelp[platform as keyof typeof syntaxHelp].map((item, index) => (
            <div key={index} className="syntax-tag" onClick={() => handleSelect(item.label)}>
              <span style={{ color: 'var(--accent-cyan)', marginRight: 8, fontWeight: 'bold' }}>{item.label}</span>
              <span style={{ color: 'var(--text-secondary)' }}>{item.description}</span>
            </div>
          ))}
        </div>
      </Card>

      {/* 搜索进度弹窗 */}
      <ProgressModal
        open={searchModalOpen}
        title="资产查询"
        status={searchStatus}
        percent={searchPercent}
        statusText={searchStatusText}
        logs={searchLogs}
        onClose={() => setSearchModalOpen(false)}
      />

      {/* 导出进度弹窗 */}
      <ProgressModal
        open={exportProgress.modalOpen}
        title="数据导出"
        status={exportProgress.status}
        percent={exportProgress.percent}
        statusText={exportProgress.statusText}
        logs={exportProgress.logs}
        summary={exportProgress.summary}
        onClose={() => exportProgress.setModalOpen(false)}
      />
    </Card>
  );
};

export default AssetQuery;
