"use client";

import type { ControlNodeSession } from "./api/auth/[...nextauth]/route"

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
import { CaretDownIcon } from "@radix-ui/react-icons";

import { ThemeContext } from "./ThemeController";

export default function ProfileDropdown() {
  const { mobile } = useContext(ThemeContext);

  const { data: session, status } = useSession();

  console.log(session, status, (session as ControlNodeSession)?.sub);

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
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
            {mobile ? undefined : (
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
            )}
            <CaretDownIcon />
          </Flex>
        </Button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item shortcut="⌘ E">Edit</DropdownMenu.Item>
        <DropdownMenu.Item shortcut="⌘ D">Duplicate</DropdownMenu.Item>
        <DropdownMenu.Separator />
        <DropdownMenu.Item shortcut="⌘ N">Archive</DropdownMenu.Item>

        <DropdownMenu.Sub>
          <DropdownMenu.SubTrigger>More</DropdownMenu.SubTrigger>
          <DropdownMenu.SubContent>
            <DropdownMenu.Item>Move to project…</DropdownMenu.Item>
            <DropdownMenu.Item>Move to folder…</DropdownMenu.Item>

            <DropdownMenu.Separator />
            <DropdownMenu.Item>Advanced options…</DropdownMenu.Item>
          </DropdownMenu.SubContent>
        </DropdownMenu.Sub>

        <DropdownMenu.Separator />
        <DropdownMenu.Item>Share</DropdownMenu.Item>
        <DropdownMenu.Item>Add to favorites</DropdownMenu.Item>
        <DropdownMenu.Separator />
        <DropdownMenu.Item shortcut="⌘ ⌫" color="red">
          Delete
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  );
}
