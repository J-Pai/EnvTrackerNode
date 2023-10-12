import "./globals.css";
import type { Metadata } from "next";

import ThemeController from "./ThemeController";

export const metadata: Metadata = {
  title: "Console Node",
  description: "Console for EnvTrackerNode",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <ThemeController>{children}</ThemeController>
      </body>
    </html>
  );
}
