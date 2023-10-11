'use client'

import { Flex, Text, Button } from '@radix-ui/themes'

export default function Home() {
  function sayHello() {
    console.log("HELLO WORLD");
  }

  return (
    <Flex direction="column" gap="2">
      <Text>Hello from Radix Themes :)</Text>
      <Button onClick={sayHello}>Let&apos;s go</Button>
    </Flex>
  )
}
