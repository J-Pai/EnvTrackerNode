"use client";

import { Button, Flex, Section, Text } from "@radix-ui/themes";
import { useState } from "react";

import * as ControlNode from "./ControlNode";

export default function Home() {
  const [echoMessage, setEchoMessage] = useState<string>("");
  const [count, setCount] = useState<number>(0);

  const sendEchoRequestBtnClick = async () => {
    const resp = await ControlNode.postEcho(`Echo ${count + 1}`);
    setEchoMessage(resp.echo);
    setCount(count + 1);
  };

  const resetCountBtnClick = async () => {
    const resp = await ControlNode.postEcho(`Echo 0`);
    setEchoMessage(resp.echo);
    setCount(0);
  };

  return (
    <Flex direction="column">
      <Section>
        <Flex direction="column">
          <Flex direction="row" gap="3">
            <Button onClick={() => sendEchoRequestBtnClick()}>
              Send Echo Request
            </Button>
            <Button onClick={() => resetCountBtnClick()}>
              Reset Count
            </Button>
          </Flex>
          <Text>FROM ControlNode: {echoMessage}</Text>
        </Flex>
      </Section>
      <Section>
        <Text>Section 1</Text>
      </Section>
      <Section>
        <Text>Section 2</Text>
      </Section>
      <Section>
        <Text>Section 3</Text>
      </Section>
      <Section>
        <Text>Section 3</Text>
      </Section>
      <Section>
        <Text>Section 3</Text>
      </Section>
    </Flex>
  );
}
