"use client";

import { GearIcon } from "@radix-ui/react-icons";
import { Box, Flex, Heading } from "@radix-ui/themes";
import { SessionProvider } from "next-auth/react";

import { ToggleThemeButton } from "./Main";
import ProfileDropdown from "./ProfileDropdown";

export default function NavBar() {
  const handleTitleBarClick = () => {
    console.log("Redirect to home");
  };

  return (
    <SessionProvider>
      <Box
        className="NavBar"
        width="100%"
        p="2"
        style={{
          backgroundColor: "var(--gray-6)",
        }}
      >
        <Flex gap="3" width="auto" align="center" justify="start">
          <Flex width="100%" gap="2" onClick={handleTitleBarClick}>
            <Box width="5" height="5" mt="1" mx="1">
              <GearIcon height="30" width="30" />
            </Box>
            <Heading mt="1">Control Node</Heading>
          </Flex>
          <Flex gap="5" width="5" align="center" justify="end">
            <ToggleThemeButton />
            <ProfileDropdown />
          </Flex>
        </Flex>
      </Box>
    </SessionProvider>
  );
}
