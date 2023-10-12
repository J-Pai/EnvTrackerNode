"use client";

import { Card, Flex, Heading } from "@radix-ui/themes";

import { useContext } from "react";

import { ThemeContext, ToggleThemeButton } from "./ThemeController";
import ProfileDropdown from "./ProfileDropdown";

export default function NavBar() {
  const { mobile } = useContext(ThemeContext);

  return (
    <Flex gap="3" align="center" justify="start">
      <Flex width="100%">
        <Heading mt="1">Control Node</Heading>
      </Flex>
      <Flex gap="5" width="100%" align="center" justify="end">
        {mobile ? undefined : <ToggleThemeButton />}
        <ProfileDropdown />
      </Flex>
    </Flex>
  );
}
