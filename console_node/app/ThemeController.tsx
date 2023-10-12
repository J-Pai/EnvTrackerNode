'use client';

import type { ThemeOptions } from '@radix-ui/themes';

import { createContext, useContext, useEffect, useState } from 'react';
import { MoonIcon, SunIcon } from '@radix-ui/react-icons';
import { IconButton, Theme } from '@radix-ui/themes';

export const ThemeContext = createContext({
  theme: '',
  setTheme: (_: string) => {},
});

export default function ThemeController({
  children,
}: {
  children: React.ReactNode;
}) {
  const [theme, setTheme] = useState('dark');

  useEffect(() => {
    setTheme(localStorage.getItem('theme') || 'light');
  }, []);

  return (
    <ThemeContext.Provider value={{ theme, setTheme }}>
      <Theme appearance={theme as ThemeOptions['appearance']}>{children}</Theme>
    </ThemeContext.Provider>
  );
}

export function ToggleThemeButton() {
  const { theme, setTheme } = useContext(ThemeContext);

  const onClick = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
  };

  return (
    <IconButton variant='ghost' color='gray' onClick={onClick} size='4'>
      {theme == 'light' ? (
        <SunIcon width='24' height='24' />
      ) : (
        <MoonIcon width='24' height='24' />
      )}
    </IconButton>
  );
}
