import { FaMarkdown, FaCode } from "react-icons/fa";
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
  const editorBg = useColorModeValue("gray.50", "gray.900");
  const borderColor = useColorModeValue("gray.200", "gray.600");

  // Check if mermaid code has content
  const hasContent = !!mermaidCode && mermaidCode.trim().length > 0;

  return (
    <Box
      w="100%"
      overflow="auto"
      borderWidth="1px"
      borderColor={borderColor}
      borderRadius="md"
      bg={editorBg}
      p={4}
      mb={4}
    >
      {hasContent ? (
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
          <Box textAlign="center" maxW="md" p={6}>
            <Icon asChild fontSize="6xl" mb={4} color="gray.400">
              <FaCode />
            </Icon>
            <Text fontSize="lg" fontWeight="semibold" color="gray.600" mb={2}>
              No Mermaid Code to Display
            </Text>
            <Text fontSize="sm" color="gray.500">
              {isWasmLoaded
                ? "Enter Python code with classes in the editor to generate the corresponding Mermaid diagram syntax."
                : "Loading..."}
            </Text>
          </Box>
        </Box>
      )}
    </Box>
  );
}
