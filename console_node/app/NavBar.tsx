"use client";

import { GearIcon } from "@radix-ui/react-icons";
import { Box, Flex, Heading } from "@radix-ui/themes";
import { SessionProvider } from "next-auth/react";

import { useContext } from "react";

import { ThemeContext, ToggleThemeButton } from "./ThemeController";
import ProfileDropdown from "./ProfileDropdown";

export default function NavBar() {
  const { mobile } = useContext(ThemeContext);

  const handleTitleBarClick = () => {
    console.log("Redirect to home");
  };

  return (
    <SessionProvider>
      <Flex gap="3" align="center" justify="start">
        <Flex width="100%" gap="2" onClick={handleTitleBarClick}>
          <Box width="5" height="5" mt="1" mx="1">
            <GearIcon height="30" width="30" />
          </Box>
          <Heading mt="1">Control Node</Heading>
        </Flex>
        <Flex gap="5" width="100%" align="center" justify="end">
          {mobile ? undefined : <ToggleThemeButton />}
          <ProfileDropdown />
        </Flex>
      </Flex>
    </SessionProvider>
  );
}
