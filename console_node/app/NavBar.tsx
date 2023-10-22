"use client";

import { GearIcon } from "@radix-ui/react-icons";
import { Container, Box, Flex, Heading, Separator } from "@radix-ui/themes";
import { SessionProvider } from "next-auth/react";

import { useContext } from "react";

import { ThemeContext, ToggleThemeButton } from "./Main";
import ProfileDropdown from "./ProfileDropdown";

export default function NavBar() {
  const { mobile } = useContext(ThemeContext);

  const handleTitleBarClick = () => {
    console.log("Redirect to home");
  };

  return (
    <SessionProvider>
      <Box width="100%" position="sticky" p="2">
        <Flex gap="3" width="auto" align="center" justify="start" pb="2">
          <Flex width="100%" gap="2" onClick={handleTitleBarClick}>
            <Box width="5" height="5" mt="1" mx="1">
              <GearIcon height="30" width="30" />
            </Box>
            <Heading mt="1">Control Node</Heading>
          </Flex>
          <Flex gap="5" width="5" align="center" justify="end">
            {mobile ? undefined : <ToggleThemeButton />}
            <ProfileDropdown />
          </Flex>
        </Flex>
        <Separator size="4" />
      </Box>
    </SessionProvider>
  );
}
