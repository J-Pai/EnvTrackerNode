import "./globals.css";
import type { Metadata } from "next";

import Main from "./Main";

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
      <body style={{ margin: "0", overflow: "hidden" }}>
        <Main>{children}</Main>
      </body>
    </html>
  );
}
