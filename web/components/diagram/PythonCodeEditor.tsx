import { useColorMode } from "@/components/ui/color-mode";
import { Box, VStack, HStack, Text } from "@chakra-ui/react";
import Editor from "@monaco-editor/react";
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
  const { colorMode } = useColorMode();
  const borderColor = colorMode === "light" ? "gray.200" : "gray.600";

  return (
    <VStack w={`${width}%`} h="100%" px={4} gap={4} align="stretch" borderRightWidth="0">
      <HStack justify="space-between" align="center" minH="41px">
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
        overflow="hidden"
        borderWidth="1px"
        borderStyle="solid"
        borderColor={borderColor}
        borderRadius="md"
        position="relative"
        mb={4}
      >
        <Editor
          height="100%"
          defaultLanguage="python"
          value={code}
          onChange={(value) => onCodeChange(value || "")}
          theme={colorMode === "light" ? "vs" : "vs-dark"}
          options={{
            minimap: { enabled: false },
            fontSize: 14,
            fontFamily: '"Fira Code", "Fira Mono", Consolas, Menlo, Courier, monospace',
            scrollBeyondLastLine: false,
            automaticLayout: true,
            tabSize: 4,
            wordWrap: "off",
            lineNumbers: "on",
            glyphMargin: false,
            folding: true,
            lineDecorationsWidth: 0,
            lineNumbersMinChars: 3,
            renderLineHighlight: "all",
            scrollbar: {
              verticalScrollbarSize: 10,
              horizontalScrollbarSize: 10,
            },
          }}
        />
      </Box>

      {!isWasmLoaded && !error && <LoadingIndicator />}
      {error && <ErrorDisplay error={error} />}
    </VStack>
  );
}
