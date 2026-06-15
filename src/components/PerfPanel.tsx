import React, { useEffect, useMemo, useState } from 'react';
import {
  Badge,
  Button,
  Drawer,
  Empty,
  Input,
  List,
  Space,
  Tag,
  Typography,
  message,
} from 'antd';
import {
  BarChartOutlined,
  ClearOutlined,
  CopyOutlined,
  ThunderboltOutlined,
} from '@ant-design/icons';
import {
  clearPerfEntries,
  getPerfEntries,
  isPerfEnabled,
  subscribePerfEntries,
  type CompletedPerfEntry,
} from '../utils/perf';

const { Text } = Typography;
const RECENT_SAMPLE_SIZE = 10;
const eventDisplayName: Record<string, string> = {
  'app-bootstrap': '应用启动',
  'view-activate': '模块切换',
  'view-chunk-load': '模块加载',
  'asset-search': '资产查询',
  'asset-export': '数据导出',
};

const averageDuration = (entries: CompletedPerfEntry[]) => {
  if (entries.length === 0) {
    return 0;
  }

  const total = entries.reduce((sum, entry) => sum + entry.duration, 0);
  return Math.round((total / entries.length) * 100) / 100;
};

const percentileDuration = (entries: CompletedPerfEntry[], percentile: number) => {
  if (entries.length === 0) {
    return 0;
  }

  const sorted = [...entries].sort((a, b) => a.duration - b.duration);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((percentile / 100) * sorted.length) - 1)
  );
  return sorted[index].duration;
};

const getSlowThreshold = (name: string) => {
  switch (name) {
    case 'app-bootstrap':
      return 2500;
    case 'view-activate':
      return 800;
    case 'view-chunk-load':
      return 500;
    case 'asset-search':
      return 1500;
    case 'asset-export':
      return 3000;
    default:
      return 1200;
  }
};

const isSlowEntry = (entry: CompletedPerfEntry) => entry.duration >= getSlowThreshold(entry.name);

const formatMetaSummary = (entry: CompletedPerfEntry) => {
  const keys = ['view', 'platform', 'source', 'exportType', 'status', 'initialView'];
  const summary = keys
    .filter((key) => entry.meta[key] != null)
    .map((key) => `${key}=${String(entry.meta[key])}`)
    .join(' · ');

  return summary || '无关键标签';
};

const getDurationColor = (duration: number) => {
  if (duration >= 2000) return 'error';
  if (duration >= 800) return 'warning';
  if (duration >= 300) return 'gold';
  return 'success';
};

const formatDuration = (value: number) => `${Math.round(value * 100) / 100} ms`;

const getRecentStats = (entries: CompletedPerfEntry[], name: string) => {
  const recentEntries = entries.filter((entry) => entry.name === name).slice(-RECENT_SAMPLE_SIZE);
  return {
    count: recentEntries.length,
    avg: averageDuration(recentEntries),
    p50: percentileDuration(recentEntries, 50),
    p95: percentileDuration(recentEntries, 95),
  };
};

type GroupedPerfStat = {
  name: string;
  count: number;
  slowCount: number;
  latest: CompletedPerfEntry;
  max: CompletedPerfEntry;
  avgAll: number;
  recent: ReturnType<typeof getRecentStats>;
};

type HotspotStat = {
  key: string;
  name: string;
  label: string;
  filterValue: string;
  count: number;
  slowCount: number;
  latest: CompletedPerfEntry;
  max: CompletedPerfEntry;
  avgAll: number;
  recent: {
    count: number;
    avg: number;
    p95: number;
  };
};

const getEntryHotspot = (entry: CompletedPerfEntry) => {
  switch (entry.name) {
    case 'app-bootstrap': {
      const initialView = String(entry.meta.initialView || 'unknown');
      return {
        key: `${entry.name}|initial:${initialView}`,
        label: `initialView=${initialView}`,
        filterValue: initialView,
      };
    }
    case 'view-activate':
    case 'view-chunk-load': {
      const view = String(entry.meta.view || 'unknown');
      return {
        key: `${entry.name}|view:${view}`,
        label: `view=${view}`,
        filterValue: view,
      };
    }
    case 'asset-search': {
      const platform = String(entry.meta.platform || 'unknown');
      const aggregated = entry.meta.aggregated ? '聚合' : '单平台';
      return {
        key: `${entry.name}|platform:${platform}|aggregated:${aggregated}`,
        label: `platform=${platform} · ${aggregated}`,
        filterValue: platform,
      };
    }
    case 'asset-export': {
      const source = String(entry.meta.source || 'unknown');
      const platform = String(entry.meta.platform || 'unknown');
      const exportType = entry.meta.exportType ? ` · type=${String(entry.meta.exportType)}` : '';
      return {
        key: `${entry.name}|source:${source}|platform:${platform}|type:${String(entry.meta.exportType || '')}`,
        label: `source=${source} · platform=${platform}${exportType}`,
        filterValue: `${source} ${platform} ${String(entry.meta.exportType || '')}`.trim(),
      };
    }
    default: {
      const summary = formatMetaSummary(entry);
      return {
        key: `${entry.name}|meta:${summary}`,
        label: summary,
        filterValue: summary,
      };
    }
  }
};

const PerfPanel: React.FC = () => {
  const [open, setOpen] = useState(false);
  const [filter, setFilter] = useState('');
  const [entries, setEntries] = useState<CompletedPerfEntry[]>(() => getPerfEntries());

  useEffect(() => {
    if (!isPerfEnabled()) {
      return;
    }

    return subscribePerfEntries(() => {
      setEntries(getPerfEntries());
    });
  }, []);

  const filteredEntries = useMemo(() => {
    const keyword = filter.trim().toLowerCase();
    const ordered = [...entries].reverse();
    if (!keyword) {
      return ordered;
    }

    return ordered.filter((entry) => {
      if (entry.name.toLowerCase().includes(keyword)) {
        return true;
      }

      return JSON.stringify(entry.meta).toLowerCase().includes(keyword);
    });
  }, [entries, filter]);

  const summary = useMemo(() => {
    const bootstrap = entries.filter((entry) => entry.name === 'app-bootstrap');
    const slowEntries = [...entries].filter(isSlowEntry).slice(-8).reverse();

    return {
      bootstrap: bootstrap.at(-1),
      viewActivation: getRecentStats(entries, 'view-activate'),
      search: getRecentStats(entries, 'asset-search'),
      export: getRecentStats(entries, 'asset-export'),
      slowEntries,
    };
  }, [entries]);

  const groupedStats = useMemo<GroupedPerfStat[]>(() => {
    const groups = new Map<string, CompletedPerfEntry[]>();

    entries.forEach((entry) => {
      const current = groups.get(entry.name);
      if (current) {
        current.push(entry);
      } else {
        groups.set(entry.name, [entry]);
      }
    });

    return Array.from(groups.entries())
      .map(([name, groupEntries]) => {
        const latest = [...groupEntries].sort((a, b) => b.recordedAt - a.recordedAt)[0];
        const max = [...groupEntries].sort((a, b) => b.duration - a.duration)[0];
        return {
          name,
          count: groupEntries.length,
          slowCount: groupEntries.filter(isSlowEntry).length,
          latest,
          max,
          avgAll: averageDuration(groupEntries),
          recent: getRecentStats(groupEntries, name),
        };
      })
      .sort((a, b) => b.max.duration - a.max.duration || b.latest.recordedAt - a.latest.recordedAt);
  }, [entries]);

  const hotspotStats = useMemo<HotspotStat[]>(() => {
    const groups = new Map<string, { name: string; label: string; filterValue: string; entries: CompletedPerfEntry[] }>();

    entries.forEach((entry) => {
      const hotspot = getEntryHotspot(entry);
      const existing = groups.get(hotspot.key);
      if (existing) {
        existing.entries.push(entry);
        return;
      }

      groups.set(hotspot.key, {
        name: entry.name,
        label: hotspot.label,
        filterValue: hotspot.filterValue,
        entries: [entry],
      });
    });

    return Array.from(groups.entries())
      .map(([key, group]) => {
        const latest = [...group.entries].sort((a, b) => b.recordedAt - a.recordedAt)[0];
        const max = [...group.entries].sort((a, b) => b.duration - a.duration)[0];
        const recentEntries = group.entries.slice(-RECENT_SAMPLE_SIZE);

        return {
          key,
          name: group.name,
          label: group.label,
          filterValue: group.filterValue,
          count: group.entries.length,
          slowCount: group.entries.filter(isSlowEntry).length,
          latest,
          max,
          avgAll: averageDuration(group.entries),
          recent: {
            count: recentEntries.length,
            avg: averageDuration(recentEntries),
            p95: percentileDuration(recentEntries, 95),
          },
        };
      })
      .sort((a, b) => b.max.duration - a.max.duration || b.avgAll - a.avgAll)
      .slice(0, 8);
  }, [entries]);

  if (!isPerfEnabled()) {
    return null;
  }

  return (
    <>
      <div
        style={{
          position: 'fixed',
          right: 20,
          bottom: 20,
          zIndex: 1100,
        }}
      >
        <Badge count={entries.length} size="small">
          <Button
            type="primary"
            icon={<BarChartOutlined />}
            onClick={() => setOpen(true)}
            style={{
              borderRadius: 999,
              boxShadow: '0 10px 28px rgba(0,0,0,0.22)',
            }}
          >
            Perf
          </Button>
        </Badge>
      </div>

      <Drawer
        title="性能面板"
        width={480}
        open={open}
        onClose={() => setOpen(false)}
        extra={
          <Space>
            <Button
              size="small"
              icon={<CopyOutlined />}
              onClick={async () => {
                await navigator.clipboard.writeText(JSON.stringify(entries, null, 2));
                message.success('性能数据已复制');
              }}
            >
              复制
            </Button>
            <Button
              size="small"
              icon={<ClearOutlined />}
              onClick={() => {
                clearPerfEntries();
                setEntries([]);
              }}
            >
              清空
            </Button>
          </Space>
        }
      >
        <Space direction="vertical" size={12} style={{ width: '100%' }}>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, minmax(0, 1fr))', gap: 8 }}>
            <div className="glass-effect" style={{ padding: 12, borderRadius: 10 }}>
              <Text type="secondary">最近启动</Text>
              <div style={{ marginTop: 4, fontWeight: 600 }}>
                {summary.bootstrap ? formatDuration(summary.bootstrap.duration) : '-'}
              </div>
              <div style={{ marginTop: 4 }}>
                <Text type="secondary">latest</Text>
              </div>
            </div>
            <div className="glass-effect" style={{ padding: 12, borderRadius: 10 }}>
              <Text type="secondary">切换近 10 次</Text>
              <div style={{ marginTop: 4, fontWeight: 600 }}>
                {summary.viewActivation.count ? formatDuration(summary.viewActivation.avg) : '-'}
              </div>
              <div style={{ marginTop: 4 }}>
                <Text type="secondary">
                  p50 {summary.viewActivation.count ? formatDuration(summary.viewActivation.p50) : '-'}
                </Text>
                <Text type="secondary" style={{ marginLeft: 8 }}>
                  p95 {summary.viewActivation.count ? formatDuration(summary.viewActivation.p95) : '-'}
                </Text>
              </div>
            </div>
            <div className="glass-effect" style={{ padding: 12, borderRadius: 10 }}>
              <Text type="secondary">查询近 10 次</Text>
              <div style={{ marginTop: 4, fontWeight: 600 }}>
                {summary.search.count ? formatDuration(summary.search.avg) : '-'}
              </div>
              <div style={{ marginTop: 4 }}>
                <Text type="secondary">
                  p50 {summary.search.count ? formatDuration(summary.search.p50) : '-'}
                </Text>
                <Text type="secondary" style={{ marginLeft: 8 }}>
                  p95 {summary.search.count ? formatDuration(summary.search.p95) : '-'}
                </Text>
              </div>
            </div>
            <div className="glass-effect" style={{ padding: 12, borderRadius: 10 }}>
              <Text type="secondary">导出近 10 次</Text>
              <div style={{ marginTop: 4, fontWeight: 600 }}>
                {summary.export.count ? formatDuration(summary.export.avg) : '-'}
              </div>
              <div style={{ marginTop: 4 }}>
                <Text type="secondary">
                  p50 {summary.export.count ? formatDuration(summary.export.p50) : '-'}
                </Text>
                <Text type="secondary" style={{ marginLeft: 8 }}>
                  p95 {summary.export.count ? formatDuration(summary.export.p95) : '-'}
                </Text>
              </div>
            </div>
          </div>

          {groupedStats.length > 0 && (
            <div
              className="glass-effect"
              style={{ padding: 12, borderRadius: 10 }}
            >
              <Space direction="vertical" size={10} style={{ width: '100%' }}>
                <Space style={{ justifyContent: 'space-between', width: '100%' }}>
                  <Text strong>事件类型聚合</Text>
                  <Text type="secondary">{groupedStats.length} 类</Text>
                </Space>
                {groupedStats.map((group) => (
                  <div
                    key={`group-${group.name}`}
                    style={{
                      padding: 10,
                      borderRadius: 10,
                      border: group.slowCount > 0
                        ? '1px solid rgba(255, 77, 79, 0.22)'
                        : '1px solid rgba(255,255,255,0.08)',
                      background: group.slowCount > 0
                        ? 'rgba(255, 77, 79, 0.05)'
                        : 'rgba(255,255,255,0.02)',
                    }}
                  >
                    <Space direction="vertical" size={8} style={{ width: '100%' }}>
                      <Space wrap style={{ justifyContent: 'space-between', width: '100%' }}>
                        <Space wrap>
                          <Tag
                            color="blue"
                            style={{ cursor: 'pointer' }}
                            onClick={() => setFilter(group.name)}
                          >
                            {eventDisplayName[group.name] || group.name}
                          </Tag>
                          <Tag>{group.count} 次</Tag>
                          {group.slowCount > 0 && <Tag color="error">慢样本 {group.slowCount}</Tag>}
                        </Space>
                        <Text type="secondary">
                          最近 {new Date(group.latest.recordedAt).toLocaleTimeString()}
                        </Text>
                      </Space>
                      <Space wrap size={[8, 8]}>
                        <Tag>总体均值 {formatDuration(group.avgAll)}</Tag>
                        <Tag>近10次均值 {group.recent.count ? formatDuration(group.recent.avg) : '-'}</Tag>
                        <Tag>近10次 p95 {group.recent.count ? formatDuration(group.recent.p95) : '-'}</Tag>
                        <Tag color={getDurationColor(group.max.duration)}>
                          最慢 {formatDuration(group.max.duration)}
                        </Tag>
                      </Space>
                      <Text type="secondary">
                        最慢样本: {formatMetaSummary(group.max)}
                      </Text>
                    </Space>
                  </div>
                ))}
              </Space>
            </div>
          )}

          {hotspotStats.length > 0 && (
            <div
              className="glass-effect"
              style={{ padding: 12, borderRadius: 10 }}
            >
              <Space direction="vertical" size={10} style={{ width: '100%' }}>
                <Space style={{ justifyContent: 'space-between', width: '100%' }}>
                  <Text strong>热点维度定位</Text>
                  <Text type="secondary">Top {hotspotStats.length}</Text>
                </Space>
                {hotspotStats.map((hotspot) => (
                  <div
                    key={hotspot.key}
                    style={{
                      padding: 10,
                      borderRadius: 10,
                      border: hotspot.slowCount > 0
                        ? '1px solid rgba(255, 77, 79, 0.22)'
                        : '1px solid rgba(255,255,255,0.08)',
                      background: hotspot.slowCount > 0
                        ? 'rgba(255, 77, 79, 0.05)'
                        : 'rgba(255,255,255,0.02)',
                    }}
                  >
                    <Space direction="vertical" size={8} style={{ width: '100%' }}>
                      <Space wrap style={{ justifyContent: 'space-between', width: '100%' }}>
                        <Space wrap>
                          <Tag color="geekblue">{eventDisplayName[hotspot.name] || hotspot.name}</Tag>
                          <Tag
                            style={{ cursor: 'pointer' }}
                            onClick={() => setFilter(hotspot.filterValue)}
                          >
                            {hotspot.label}
                          </Tag>
                          <Tag>{hotspot.count} 次</Tag>
                          {hotspot.slowCount > 0 && <Tag color="error">慢样本 {hotspot.slowCount}</Tag>}
                        </Space>
                        <Text type="secondary">
                          最近 {new Date(hotspot.latest.recordedAt).toLocaleTimeString()}
                        </Text>
                      </Space>
                      <Space wrap size={[8, 8]}>
                        <Tag>总体均值 {formatDuration(hotspot.avgAll)}</Tag>
                        <Tag>近10次均值 {hotspot.recent.count ? formatDuration(hotspot.recent.avg) : '-'}</Tag>
                        <Tag>近10次 p95 {hotspot.recent.count ? formatDuration(hotspot.recent.p95) : '-'}</Tag>
                        <Tag color={getDurationColor(hotspot.max.duration)}>
                          最慢 {formatDuration(hotspot.max.duration)}
                        </Tag>
                      </Space>
                      <Text type="secondary">
                        最慢样本: {formatMetaSummary(hotspot.max)}
                      </Text>
                    </Space>
                  </div>
                ))}
              </Space>
            </div>
          )}

          {summary.slowEntries.length > 0 && (
            <div
              className="glass-effect"
              style={{
                padding: 12,
                borderRadius: 10,
                border: '1px solid rgba(255, 77, 79, 0.28)',
                background: 'rgba(255, 77, 79, 0.08)',
              }}
            >
              <Space direction="vertical" size={10} style={{ width: '100%' }}>
                <Space>
                  <ThunderboltOutlined style={{ color: '#ff4d4f' }} />
                  <Text strong>慢操作高亮</Text>
                  <Tag color="error">{summary.slowEntries.length}</Tag>
                </Space>
                {summary.slowEntries.map((entry) => (
                  <div
                    key={`slow-${entry.id}`}
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      gap: 12,
                    }}
                  >
                    <Space wrap size={[6, 6]}>
                      <Tag color="volcano">{entry.name}</Tag>
                      <Tag color="error">{formatDuration(entry.duration)}</Tag>
                      {entry.meta.view != null && <Tag>view: {String(entry.meta.view)}</Tag>}
                      {entry.meta.platform != null && <Tag>platform: {String(entry.meta.platform)}</Tag>}
                    </Space>
                    <Text type="secondary">{new Date(entry.recordedAt).toLocaleTimeString()}</Text>
                  </div>
                ))}
              </Space>
            </div>
          )}

          <Input
            value={filter}
            onChange={(event) => setFilter(event.target.value)}
            placeholder="筛选名称或元数据"
            allowClear
          />

          {filteredEntries.length === 0 ? (
            <Empty description="暂无性能记录" />
          ) : (
            <List
              dataSource={filteredEntries}
              renderItem={(entry) => (
                <List.Item
                  style={isSlowEntry(entry) ? {
                    padding: 12,
                    borderRadius: 10,
                    border: '1px solid rgba(255, 77, 79, 0.24)',
                    background: 'rgba(255, 77, 79, 0.06)',
                    marginBottom: 8,
                  } : undefined}
                >
                  <Space direction="vertical" size={6} style={{ width: '100%' }}>
                    <Space wrap style={{ justifyContent: 'space-between', width: '100%' }}>
                      <Space wrap>
                        <Tag color="blue">{entry.name}</Tag>
                        <Tag color={getDurationColor(entry.duration)}>{entry.duration} ms</Tag>
                        {isSlowEntry(entry) && <Tag color="error">慢</Tag>}
                      </Space>
                      <Text type="secondary">
                        {new Date(entry.recordedAt).toLocaleTimeString()}
                      </Text>
                    </Space>
                    <Space wrap size={[6, 6]}>
                      {Object.entries(entry.meta).map(([key, value]) => (
                        <Tag key={`${entry.id}-${key}`}>
                          {key}: {String(value)}
                        </Tag>
                      ))}
                    </Space>
                  </Space>
                </List.Item>
              )}
            />
          )}
        </Space>
      </Drawer>
    </>
  );
};

export default PerfPanel;
