"use client";

import type { ThemeOptions } from "@radix-ui/themes";

import { createContext, useContext, useEffect, useState } from "react";
import { MoonIcon, SunIcon } from "@radix-ui/react-icons";
import { Box, IconButton, Theme } from "@radix-ui/themes";

import NavBar from "./NavBar";

export const ThemeContext = createContext({
  theme: "",
  setTheme: (_: string) => {},
  mobile: false,
});

export default function Main({ children }: { children: React.ReactNode }) {
  const [theme, setTheme] = useState<string>("dark");
  const [mobile, setMobile] = useState<boolean>(false);

  const handleWindowSizingChange = () => {
    if (window.innerWidth < 768) {
      setMobile(true);
    } else {
      setMobile(false);
    }
  };

  useEffect(() => {
    window.addEventListener("resize", handleWindowSizingChange);
    handleWindowSizingChange();
    setTheme(localStorage.getItem("theme") || "light");
  }, []);

  return (
    <ThemeContext.Provider value={{ theme, setTheme, mobile }}>
      <Theme
        appearance={theme as ThemeOptions["appearance"]}
        accentColor="gray"
        style={{ margin: "0" }}
      >
        <NavBar />
        {children}
      </Theme>
    </ThemeContext.Provider>
  );
}

export function ToggleThemeButton() {
  const { theme, setTheme } = useContext(ThemeContext);

  const onClick = () => {
    const newTheme = theme === "light" ? "dark" : "light";
    setTheme(newTheme);
    localStorage.setItem("theme", newTheme);
  };

  return (
    <IconButton variant="ghost" color="gray" onClick={onClick} size="4">
      {theme == "light" ? (
        <SunIcon width="24" height="24" />
      ) : (
        <MoonIcon width="24" height="24" />
      )}
    </IconButton>
  );
}
