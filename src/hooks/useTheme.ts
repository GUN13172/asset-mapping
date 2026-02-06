import { useState, useEffect } from 'react';

export type ThemeType = 'light' | 'dark' | 'system';

export const useTheme = () => {
  const [theme, setThemeState] = useState<ThemeType>(() => {
    // 初始化时从 localStorage 读取
    const savedTheme = localStorage.getItem('app-theme') as ThemeType;
    console.log('🎨 初始化主题:', savedTheme || 'dark');
    return savedTheme || 'dark'; // 默认使用深色主题
  });

  useEffect(() => {
    // 应用当前主题
    console.log('🎨 应用主题:', theme);
    applyTheme(theme);
  }, [theme]);

  const applyTheme = (newTheme: ThemeType) => {
    let effectiveTheme = newTheme;

    if (newTheme === 'system') {
      // 检测系统主题
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      effectiveTheme = prefersDark ? 'dark' : 'light';
      console.log('🎨 系统主题检测:', effectiveTheme);
    }

    // 应用主题到HTML元素
    console.log('🎨 设置 data-theme 属性:', effectiveTheme);
    document.documentElement.setAttribute('data-theme', effectiveTheme);
    
    // 验证属性是否设置成功
    const actualTheme = document.documentElement.getAttribute('data-theme');
    console.log('🎨 验证 data-theme 属性:', actualTheme);
    
    // 保存到 localStorage
    localStorage.setItem('app-theme', newTheme);
    console.log('🎨 保存主题到 localStorage:', newTheme);
  };

  const setTheme = (newTheme: ThemeType) => {
    console.log('🎨 切换主题:', theme, '->', newTheme);
    setThemeState(newTheme);  // 触发状态更新
  };

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
  };

  return { theme, setTheme, toggleTheme };
};
