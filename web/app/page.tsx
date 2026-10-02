"use client";

import { Box, Container, Flex, Heading, VStack, HStack, IconButton, Link } from "@chakra-ui/react";
import DiagramEditor from "@/components/DiagramEditor";
import { ColorModeButton } from "@/components/ui/color-mode";
import { FaGithub } from "react-icons/fa";

const PRODUCTION_URL = "https://diceroll123.github.io/pymermaider/";
const IS_PR_PREVIEW = (process.env.NEXT_PUBLIC_BASE_PATH ?? "").includes("/pr-preview/");

export default function Home() {
  return (
    <Container maxW="100vw" p={0} h="100vh">
      <VStack h="100%" gap={0}>
        <Box
          w="100%"
          p={4}
          bg={{ base: "white", _dark: "black" }}
          borderBottomWidth="1px"
          borderBottomColor="border.muted"
          flexShrink={0}
        >
          <Flex justify="space-between" align="center">
            <Box>
              <Heading size="xl">PyMermaider</Heading>
              <Box fontSize="sm" mt={1}>
                Generate Mermaid class diagrams from Python code
              </Box>
            </Box>
            <HStack gap={2}>
              {IS_PR_PREVIEW && (
                <Link href={PRODUCTION_URL} fontSize="sm">
                  Production playground
                </Link>
              )}
              <Link
                href="https://github.com/diceroll123/pymermaider"
                target="_blank"
                rel="noopener noreferrer"
                aria-label="View on GitHub"
              >
                <IconButton
                  aria-label="View on GitHub"
                  size="lg"
                  variant="ghost"
                >
                  <FaGithub size={24} />
                </IconButton>
              </Link>
              <ColorModeButton size="lg" />
            </HStack>
          </Flex>
        </Box>
        <DiagramEditor />
      </VStack>
    </Container>
  );
}
