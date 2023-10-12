import './globals.css';
import type { Metadata } from 'next';

import { Flex, Heading } from '@radix-ui/themes';

import ProfileDropdown from './ProfileDropdown';
import ThemeController, { ToggleThemeButton } from './ThemeController';

export const metadata: Metadata = {
  title: 'Console Node',
  description: 'Console for EnvTrackerNode',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang='en' suppressHydrationWarning>
      <body>
        <ThemeController>
          <Flex gap='3' align='center' justify='start'>
            <Flex width='100%'>
              <Heading>Control Node</Heading>
            </Flex>
            <Flex gap='5' width='100%' align='center' justify='end'>
              <ToggleThemeButton />
              <ProfileDropdown />
            </Flex>
          </Flex>
          {children}
        </ThemeController>
      </body>
    </html>
  );
}
