"use client";

import { useState } from "react";
import { Flex, VStack, Text, Tabs } from "@chakra-ui/react";
import { useColorMode } from "@/components/ui/color-mode";
import { DEFAULT_PYTHON_CODE } from "./diagram/types";
import { useWasm } from "./diagram/hooks/useWasm";
import { useMermaid } from "./diagram/hooks/useMermaid";
import { useSyntaxHighlighter } from "./diagram/hooks/useSyntaxHighlighter";
import { useResizablePanel } from "./diagram/hooks/useResizablePanel";
import { PythonCodeEditor } from "./diagram/PythonCodeEditor";
import { DiagramView } from "./diagram/DiagramView";
import { MermaidCodeView } from "./diagram/MermaidCodeView";
import { ResizableDivider } from "./diagram/ResizableDivider";

export default function DiagramEditor() {
  const [pythonCode, setPythonCode] = useState(DEFAULT_PYTHON_CODE);
  const { colorMode, mounted: themeMounted } = useColorMode();
  const [activeTab, setActiveTab] = useState("diagram");

  // Load WASM
  const { wasmRef, isWasmLoaded, error: wasmError } = useWasm();

  // Generate Mermaid diagram
  const {
    mermaidCode,
    diagramSvg,
    error: mermaidError,
    isProcessing,
  } = useMermaid({
    wasmRef,
    pythonCode,
    isWasmLoaded,
    colorMode,
    themeMounted,
  });

  // Syntax highlighting for Python editor
  const { handleCodeChange, isHighlightingEnabled, lineCount } =
    useSyntaxHighlighter({
      code: pythonCode,
      colorMode,
      themeMounted,
    });

  // Resizable panel
  const { leftPanelWidth, isDragging, handleMouseDown, resetToCenter } = useResizablePanel(50);

  // Combined error from WASM or Mermaid
  const error = wasmError || mermaidError;

  const handlePythonCodeChange = (newCode: string) => {
    const processedCode = handleCodeChange(newCode);
    setPythonCode(processedCode);
  };

  return (
    <Flex
      w="100%"
      flex={1}
      gap={0}
      position="relative"
      style={{ userSelect: isDragging ? "none" : "auto" }}
    >
      {/* Left Panel - Code Input */}
      <PythonCodeEditor
        width={leftPanelWidth}
        code={pythonCode}
        onCodeChange={handlePythonCodeChange}
        isWasmLoaded={isWasmLoaded}
        error={error}
        isProcessing={isProcessing}
        lineCount={lineCount}
        isHighlightingEnabled={isHighlightingEnabled}
      />

      {/* Resizable Divider */}
      <ResizableDivider
        isDragging={isDragging}
        onMouseDown={handleMouseDown}
        onDoubleClick={resetToCenter}
        leftPosition={leftPanelWidth}
      />

      {/* Right Panel - Tabbed View */}
      <VStack w={`${100 - leftPanelWidth}%`} h="100%" px={4} gap={4} align="stretch">
        <Tabs.Root value={activeTab} onValueChange={(e) => setActiveTab(e.value)} fitted>
          <Tabs.List>
            <Tabs.Trigger value="diagram">
              <Text fontSize="md" fontWeight="medium">
                Diagram
              </Text>
            </Tabs.Trigger>
            <Tabs.Trigger value="code">
              <Text fontSize="md" fontWeight="medium">
                Mermaid Code
              </Text>
            </Tabs.Trigger>
          </Tabs.List>
        </Tabs.Root>

        <div style={{ position: "relative", flex: 1 }}>
          <div style={{ height: "100%", display: "flex", visibility: activeTab === "diagram" ? "visible" : "hidden", position: "absolute", top: 0, left: 0, right: 0, bottom: 0 }}>
            <DiagramView
              diagramSvg={diagramSvg}
              error={error}
              isWasmLoaded={isWasmLoaded}
            />
          </div>

          <div style={{ height: "100%", display: "flex", visibility: activeTab === "code" ? "visible" : "hidden", position: "absolute", top: 0, left: 0, right: 0, bottom: 0 }}>
            <MermaidCodeView
              mermaidCode={mermaidCode}
              colorMode={colorMode}
              isWasmLoaded={isWasmLoaded}
            />
          </div>
        </div>
      </VStack>
    </Flex>
  );
}
