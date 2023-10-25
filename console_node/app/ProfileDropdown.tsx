"use client";

import { CaretDownIcon, ExitIcon } from "@radix-ui/react-icons";
import { useSession, signIn, signOut } from "next-auth/react";
import { useContext } from "react";
import {
  Avatar,
  Box,
  Button,
  DropdownMenu,
  Flex,
  Text,
} from "@radix-ui/themes";
import { FcGoogle } from "react-icons/fc";

import { ThemeContext } from "./Main";

export default function ProfileDropdown() {
  const { mobile } = useContext(ThemeContext);

  const { data: session, status } = useSession();

  let profileInfo = undefined;
  let userActionMenu = undefined;
  let triggerContents = undefined;

  if (status === "authenticated") {
    profileInfo = (
      <Flex>
        <Box>
          <Text as="div" size="2" weight="bold">
            {session?.user?.name}
          </Text>
          <Text as="div" size="2" color="gray">
            {session?.user?.email}
          </Text>
        </Box>
      </Flex>
    );
    triggerContents = (
      <Button
        variant="soft"
        style={{
          height: "fit-content",
          paddingTop: "12px",
          paddingBottom: "12px",
        }}
      >
        <Flex gap="4" align="center">
          <Avatar
            size="3"
            src={session?.user?.image || undefined}
            radius="full"
            fallback="X"
          />
          {mobile ? undefined : profileInfo}
          <CaretDownIcon />
        </Flex>
      </Button>
    );
    userActionMenu = (
      <DropdownMenu.Content
        align="end"
        style={{
          width: "",
        }}
      >
        <Button onClick={() => signOut()} size="3">
          <ExitIcon />
          Sign Out
        </Button>
      </DropdownMenu.Content>
    );
  } else if (status === "loading") {
    profileInfo = (
      <Flex>
        <Box>
          <Text as="div" size="2" weight="bold">
            Loading...
          </Text>
        </Box>
      </Flex>
    );
    triggerContents = (
      <Button
        variant="soft"
        style={{
          height: "fit-content",
          paddingTop: "12px",
          paddingBottom: "12px",
        }}
      >
        <Flex gap="4" align="center">
          <Box width="7" height="7">
            <FcGoogle style={{ width: "100%", height: "100%" }} />
          </Box>
          {mobile ? undefined : profileInfo}
          <CaretDownIcon />
        </Flex>
      </Button>
    );
  } else {
    profileInfo = (
      <Flex>
        <Box>
          <Text as="div" size="2" weight="bold">
            Log In with Google
          </Text>
        </Box>
      </Flex>
    );
    triggerContents = (
      <Button
        variant="soft"
        style={{
          height: "fit-content",
          paddingTop: "12px",
          paddingBottom: "12px",
        }}
        onClick={() => signIn("google")}
      >
        <Flex gap="4" align="center">
          <Box width="7" height="7">
            <FcGoogle style={{ width: "100%", height: "100%" }} />
          </Box>
          {mobile ? undefined : profileInfo}
        </Flex>
      </Button>
    );
  }

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>{triggerContents}</DropdownMenu.Trigger>
      {userActionMenu}
    </DropdownMenu.Root>
  );
}
