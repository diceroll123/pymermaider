import { useMemo } from "react";
import { Box, VStack, HStack, Text } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";
import Editor from "react-simple-code-editor";
import { ErrorDisplay } from "./ErrorDisplay";
import { LoadingIndicator } from "./LoadingIndicator";

interface PythonCodeEditorProps {
  width: number;
  code: string;
  onCodeChange: (code: string) => void;
  highlightCode: (code: string) => string;
  isWasmLoaded: boolean;
  error: string | null;
  isProcessing: boolean;
  lineCount: number;
  isHighlightingEnabled: boolean;
}

export function PythonCodeEditor({
  width,
  code,
  onCodeChange,
  highlightCode,
  isWasmLoaded,
  error,
  isProcessing,
  lineCount,
  isHighlightingEnabled,
}: PythonCodeEditorProps) {
  const borderColor = useColorModeValue("gray.200", "gray.600");
  const editorBg = useColorModeValue("gray.50", "gray.900");

  const editorStyle = useMemo(
    () => ({
      fontFamily:
        '"Fira Code", "Fira Mono", Consolas, Menlo, Courier, monospace',
      fontSize: 14,
      minHeight: "100%",
      outline: "none",
    }),
    []
  );

  return (
    <VStack w={`${width}%`} h="100%" p={4} gap={4} align="stretch" borderRightWidth="0">
      <HStack justify="space-between" align="center">
        <Text fontSize="lg" fontWeight="bold">
          Python Code
        </Text>
        <HStack gap={2}>
          {lineCount > 1000 && isHighlightingEnabled && (
            <Text fontSize="xs" color="gray.500">
              Large file - highlighting deferred
            </Text>
          )}
          {isProcessing && (
            <Text fontSize="sm" color="blue.600">
              Generating diagram...
            </Text>
          )}
        </HStack>
      </HStack>

      <Box
        flex={1}
        overflow="auto"
        border="1px"
        borderColor={borderColor}
        borderRadius="md"
        bg={editorBg}
        position="relative"
      >
        <Editor
          key="python-editor"
          value={code}
          onValueChange={onCodeChange}
          highlight={highlightCode}
          padding={10}
          style={editorStyle}
          placeholder="Enter your Python code here..."
          textareaId="python-code-textarea"
        />
      </Box>

      {!isWasmLoaded && !error && <LoadingIndicator />}
      {error && <ErrorDisplay error={error} />}
    </VStack>
  );
}
