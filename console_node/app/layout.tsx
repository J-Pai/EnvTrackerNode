import './globals.css'
import type { Metadata } from 'next'

import { Theme } from '@radix-ui/themes'

export const metadata: Metadata = {
  title: 'Console Node',
  description: 'Console for EnvTrackerNode',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body>
        <Theme appearance="dark">
          {children}
        </Theme>
      </body>
    </html>
  )
}
