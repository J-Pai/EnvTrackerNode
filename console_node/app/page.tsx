"use client";

import { Button, Flex, Section, Text } from "@radix-ui/themes";
import { useState } from "react";

import * as ControlNode from "./ControlNode";

export default function Home() {
  const [echoMessage, setEchoMessage] = useState<string>("");

  const sendEchoRequestButtonClick = async () => {
    const resp = await ControlNode.postEcho("Echo {}");
    setEchoMessage(resp.echo);
  };

  return (
    <Flex direction="column">
      <Section>
        <Button onClick={() => sendEchoRequestButtonClick()}>
          Send Echo Request
        </Button>
        <Text>{echoMessage}</Text>
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
