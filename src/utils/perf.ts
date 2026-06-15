export type PerfValue = string | number | boolean | null | undefined;
export type PerfMetadata = Record<string, PerfValue>;
export type PerfToken = string | null;

type PendingPerfEntry = {
  id: string;
  name: string;
  meta: PerfMetadata;
  startedAt: number;
};

export type CompletedPerfEntry = PendingPerfEntry & {
  endedAt: number;
  recordedAt: number;
  duration: number;
};

const PERF_UPDATE_EVENT = 'asset-mapping:perf-update';

declare global {
  interface Window {
    __assetMappingPerf__?: CompletedPerfEntry[];
  }
}

const pendingPerfEntries = new Map<string, PendingPerfEntry>();
let perfCounter = 0;

const roundDuration = (value: number) => Math.round(value * 100) / 100;

export const isPerfEnabled = () => {
  if (typeof window === 'undefined') {
    return false;
  }

  try {
    return import.meta.env.DEV || window.localStorage.getItem('__asset_mapping_perf') === '1';
  } catch {
    return import.meta.env.DEV;
  }
};

const getCompletedEntries = (): CompletedPerfEntry[] => {
  if (typeof window === 'undefined') {
    return [];
  }

  if (!window.__assetMappingPerf__) {
    window.__assetMappingPerf__ = [];
  }

  return window.__assetMappingPerf__;
};

const emitPerfUpdate = () => {
  if (typeof window === 'undefined') {
    return;
  }

  window.dispatchEvent(new CustomEvent(PERF_UPDATE_EVENT));
};

export const getPerfEntries = () => [...getCompletedEntries()];

export const clearPerfEntries = () => {
  if (typeof window === 'undefined') {
    return;
  }

  window.__assetMappingPerf__ = [];
  emitPerfUpdate();
};

export const subscribePerfEntries = (callback: () => void) => {
  if (typeof window === 'undefined') {
    return () => undefined;
  }

  const listener = () => callback();
  window.addEventListener(PERF_UPDATE_EVENT, listener);
  return () => {
    window.removeEventListener(PERF_UPDATE_EVENT, listener);
  };
};

export const startPerf = (name: string, meta: PerfMetadata = {}): PerfToken => {
  if (!isPerfEnabled()) {
    return null;
  }

  const id = `${name}:${++perfCounter}`;
  pendingPerfEntries.set(id, {
    id,
    name,
    meta,
    startedAt: performance.now(),
  });
  return id;
};

export const endPerf = (token: PerfToken, meta: PerfMetadata = {}) => {
  if (!token) {
    return;
  }

  const entry = pendingPerfEntries.get(token);
  if (!entry) {
    return;
  }

  pendingPerfEntries.delete(token);

  const now = performance.now();
  const completed: CompletedPerfEntry = {
    ...entry,
    meta: { ...entry.meta, ...meta },
    endedAt: now,
    recordedAt: Date.now(),
    duration: roundDuration(now - entry.startedAt),
  };

  const completedEntries = getCompletedEntries();
  completedEntries.push(completed);
  if (completedEntries.length > 200) {
    completedEntries.splice(0, completedEntries.length - 200);
  }

  console.info(`[perf] ${completed.name} ${completed.duration}ms`, completed.meta);
  emitPerfUpdate();
};

export const markPerf = (name: string, meta: PerfMetadata = {}) => {
  const token = startPerf(name, meta);
  endPerf(token);
};

export const scheduleIdle = (callback: () => void, timeout = 1500) => {
  if (typeof window === 'undefined') {
    return () => undefined;
  }

  if (window.requestIdleCallback && window.cancelIdleCallback) {
    const handle = window.requestIdleCallback(() => callback(), { timeout });
    return () => window.cancelIdleCallback?.(handle);
  }

  const timer = window.setTimeout(callback, Math.min(timeout, 300));
  return () => window.clearTimeout(timer);
};
