"use client";

import { Button, Flex, Section, Text } from "@radix-ui/themes";

export default function Home() {
  return (
    <Flex direction="column">
      <Section>
        <Button>Send Echo Request</Button>
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
