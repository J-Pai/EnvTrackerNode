'use client';

import type { ThemeOptions } from '@radix-ui/themes';

import { createContext, useContext, useEffect, useState } from 'react';
import { MoonIcon, SunIcon } from '@radix-ui/react-icons';
import { Button, Theme } from '@radix-ui/themes';

const ThemeContext = createContext({
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
    setTheme(localStorage.getItem('theme') || 'light')
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
    <Button variant='ghost' color='gray' onClick={onClick}>
      {theme == 'light' ? <SunIcon /> : <MoonIcon />}
    </Button>
  );
}
