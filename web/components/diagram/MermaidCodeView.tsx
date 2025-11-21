import { FaMarkdown } from "react-icons/fa";
import { Box, Text, IconButton, Icon } from "@chakra-ui/react";
import { CodeBlock } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";
import { shikiAdapter } from "./shiki-adapter";

interface MermaidCodeViewProps {
  mermaidCode: string;
  colorMode: string | undefined;
  isWasmLoaded: boolean;
}

export function MermaidCodeView({
  mermaidCode,
  colorMode,
  isWasmLoaded,
}: MermaidCodeViewProps) {
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const editorBg = useColorModeValue("gray.50", "gray.900");

  return (
    <Box
      h="calc(100vh - 180px)"
      overflow="auto"
      border="1px"
      borderColor={borderColor}
      borderRadius="md"
      bg={editorBg}
      p={4}
      position="relative"
    >
      {mermaidCode && mermaidCode.trim().length > 0 ? (
        <CodeBlock.AdapterProvider value={shikiAdapter}>
          <CodeBlock.Root
            code={mermaidCode}
            language="mermaid"
            meta={{ showLineNumbers: true, colorScheme: colorMode }}
          >
            <CodeBlock.Header>
              <CodeBlock.Title>
                <Icon as={FaMarkdown} />
                mermaid.md
              </CodeBlock.Title>
              <CodeBlock.CopyTrigger asChild>
                <IconButton variant="ghost" size="xs">
                  <CodeBlock.CopyIndicator />
                </IconButton>
              </CodeBlock.CopyTrigger>
            </CodeBlock.Header>
            <CodeBlock.Content>
              <CodeBlock.Code>
                <CodeBlock.CodeText />
              </CodeBlock.Code>
            </CodeBlock.Content>
          </CodeBlock.Root>
        </CodeBlock.AdapterProvider>
      ) : (
        <Box
          display="flex"
          justifyContent="center"
          alignItems="center"
          h="100%"
        >
          <Text color="gray.400">
            {isWasmLoaded
              ? "Enter Python code to see the Mermaid code"
              : "Loading..."}
          </Text>
        </Box>
      )}
    </Box>
  );
}
