"use client";

import { useState, useRef, useCallback } from "react";
import { Flex, VStack, Box, Text, Tabs, IconButton, Popover, Portal, SegmentGroup } from "@chakra-ui/react";
import { LuPanelLeftClose, LuPanelLeft, LuSettings, LuArrowDown, LuArrowUp, LuArrowRight, LuArrowLeft } from "react-icons/lu";
import { useColorMode } from "@/components/ui/color-mode";
import { DEFAULT_PYTHON_CODE, DiagramDirection } from "./diagram/types";
import { useWasm } from "./diagram/hooks/useWasm";
import { useMermaid } from "./diagram/hooks/useMermaid";
import { useSyntaxHighlighter } from "./diagram/hooks/useSyntaxHighlighter";
import { useResizablePanel } from "./diagram/hooks/useResizablePanel";
import { useFileSystem } from "./diagram/hooks/useFileSystem";
import { useGitHubRepo } from "./diagram/hooks/useGitHubRepo";
import { PythonCodeEditor } from "./diagram/PythonCodeEditor";
import { DiagramView } from "./diagram/DiagramView";
import { MermaidCodeView } from "./diagram/MermaidCodeView";
import { ResizableDivider } from "./diagram/ResizableDivider";
import { FileExplorer } from "./diagram/FileExplorer";
import { RepoLoader } from "./diagram/RepoLoader";

const SIDEBAR_WIDTH = 260;

export default function DiagramEditor() {
  const [pythonCode, setPythonCode] = useState(DEFAULT_PYTHON_CODE);
  const { colorMode, mounted: themeMounted } = useColorMode();
  const [activeTab, setActiveTab] = useState("diagram");
  const fitToWidthFnRef = useRef<(() => void) | null>(null);
  const editorPanelsRef = useRef<HTMLDivElement>(null);
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [direction, setDirection] = useState<DiagramDirection>("TB");

  // Load WASM
  const { wasmRef, isWasmLoaded, error: wasmError } = useWasm();

  // File system hooks
  const fileSystem = useFileSystem();
  const gitHubRepo = useGitHubRepo();

  // Determine which source is active
  const activeSource = fileSystem.repoSource || gitHubRepo.repoSource;
  const activeFiles = fileSystem.files.length > 0 ? fileSystem.files : gitHubRepo.files;
  const isFileLoading = fileSystem.isLoading || gitHubRepo.isLoading;
  const fileError = fileSystem.error || gitHubRepo.error;

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
    direction,
  });

  // Syntax highlighting for Python editor
  const { handleCodeChange, isHighlightingEnabled, lineCount } =
    useSyntaxHighlighter({
      code: pythonCode,
      colorMode,
      themeMounted,
    });

  // Resizable panel (relative to editor panels container)
  const { leftPanelWidth, isDragging, handleMouseDown, resetToCenter } = useResizablePanel(50, editorPanelsRef);

  // Combined error from WASM or Mermaid
  const error = wasmError || mermaidError;

  const handlePythonCodeChange = (newCode: string) => {
    const processedCode = handleCodeChange(newCode);
    setPythonCode(processedCode);
  };

  const handleFitToWidthReady = useCallback((fn: () => void) => {
    fitToWidthFnRef.current = fn;
  }, []);

  const handleDividerDoubleClick = () => {
    resetToCenter();
    if (fitToWidthFnRef.current) {
      setTimeout(() => {
        fitToWidthFnRef.current?.();
      }, 100);
    }
  };

  // Handle file selection
  const handleFileSelect = useCallback(async (path: string) => {
    setSelectedFile(path);

    // Get content based on source
    let content: string | undefined;

    if (fileSystem.repoSource) {
      content = fileSystem.getFileContent(path);
    } else if (gitHubRepo.repoSource) {
      content = await gitHubRepo.getFileContent(path);
    }

    if (content) {
      setPythonCode(content);
    }
  }, [fileSystem, gitHubRepo]);

  // Handle clearing files
  const handleClearFiles = useCallback(() => {
    fileSystem.clearFiles();
    gitHubRepo.clearFiles();
    setSelectedFile(null);
    setPythonCode(DEFAULT_PYTHON_CODE);
  }, [fileSystem, gitHubRepo]);

  // Handle drop (for local files)
  const handleDrop = useCallback(async (e: React.DragEvent) => {
    // Clear GitHub repo if switching to local
    if (gitHubRepo.repoSource) {
      gitHubRepo.clearFiles();
    }
    await fileSystem.handleDrop(e);
    setSelectedFile(null);
  }, [fileSystem, gitHubRepo]);

  // Handle file input (click to browse)
  const handleFileInput = useCallback(async (files: FileList) => {
    // Clear GitHub repo if switching to local
    if (gitHubRepo.repoSource) {
      gitHubRepo.clearFiles();
    }
    await fileSystem.handleFileInput(files);
    setSelectedFile(null);
  }, [fileSystem, gitHubRepo]);

  // Handle GitHub repo load
  const handleLoadGitHub = useCallback(async (url: string) => {
    // Clear local files if switching to GitHub
    if (fileSystem.repoSource) {
      fileSystem.clearFiles();
    }
    await gitHubRepo.loadRepo(url);
    setSelectedFile(null);
  }, [fileSystem, gitHubRepo]);

  const hasFiles = activeFiles.length > 0;

  return (
    <Flex
      w="100%"
      flex={1}
      minH={0}
      gap={0}
      position="relative"
      overflow="hidden"
      style={{ userSelect: isDragging ? "none" : "auto" }}
    >
      {/* Sidebar */}
      <Box
        w={sidebarOpen ? `${SIDEBAR_WIDTH}px` : "0px"}
        minW={sidebarOpen ? `${SIDEBAR_WIDTH}px` : "0px"}
        h="100%"
        borderRightWidth={sidebarOpen ? "1px" : "0"}
        borderColor="border.muted"
        bg="bg.subtle"
        overflow="hidden"
        transition="all 0.2s"
        position="relative"
      >
        {/* Keep content mounted but hidden when collapsed to preserve state */}
        <Flex
          direction="column"
          h="100%"
          w={`${SIDEBAR_WIDTH}px`}
          visibility={sidebarOpen ? "visible" : "hidden"}
        >
          {/* Sidebar Header */}
          <Flex
            px={3}
            py={2}
            borderBottomWidth="1px"
            borderColor="border.muted"
            align="center"
            justify="space-between"
            bg="bg.muted"
          >
            <Text fontSize="sm" fontWeight="semibold">
              Explorer
            </Text>
            <IconButton
              aria-label="Close sidebar"
              size="xs"
              variant="ghost"
              onClick={() => setSidebarOpen(false)}
            >
              <LuPanelLeftClose />
            </IconButton>
          </Flex>

          {/* Sidebar Content */}
          <Box flex={1} minH={0} overflow="auto">
            {hasFiles ? (
              <>
                <Box px={3} py={2} borderBottomWidth="1px" borderColor="border.muted">
                  <IconButton
                    aria-label="Close project"
                    size="xs"
                    variant="ghost"
                    colorPalette="red"
                    onClick={handleClearFiles}
                    w="100%"
                  >
                    Close Project
                  </IconButton>
                </Box>
                <FileExplorer
                  files={activeFiles}
                  repoSource={activeSource}
                  selectedFile={selectedFile}
                  onFileSelect={handleFileSelect}
                  onLoadChildren={gitHubRepo.repoSource ? gitHubRepo.loadChildren : undefined}
                />
              </>
            ) : (
              <RepoLoader
                isLoading={isFileLoading}
                error={fileError}
                hasFiles={hasFiles}
                onDrop={handleDrop}
                onFileInput={handleFileInput}
                onLoadGitHub={handleLoadGitHub}
                onClear={handleClearFiles}
              />
            )}
          </Box>
        </Flex>
      </Box>

      {/* Main Content Area */}
      <Flex flex={1} h="100%" position="relative" minW={0}>
        {/* Sidebar Toggle (when closed) */}
        {!sidebarOpen && (
          <Flex
            w="40px"
            minW="40px"
            h="100%"
            align="flex-start"
            justify="center"
            pt={2}
            bg="bg.subtle"
            borderRightWidth="1px"
            borderColor="border.muted"
          >
            <IconButton
              aria-label="Open sidebar"
              size="sm"
              variant="ghost"
              onClick={() => setSidebarOpen(true)}
            >
              <LuPanelLeft />
            </IconButton>
          </Flex>
        )}
        {/* Editor Panels */}
        <Flex ref={editorPanelsRef} flex={1} h="100%" position="relative" minW={0}>
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
            onDoubleClick={handleDividerDoubleClick}
            leftPosition={leftPanelWidth}
          />

          {/* Right Panel - Tabbed View */}
          <VStack w={`${100 - leftPanelWidth}%`} h="100%" px={4} gap={4} align="stretch">
            <Flex align="center" gap={2}>
              {/* Settings Popover */}
              <Popover.Root>
                <Popover.Trigger asChild>
                  <IconButton
                    aria-label="Settings"
                    size="sm"
                    variant="ghost"
                  >
                    <LuSettings />
                  </IconButton>
                </Popover.Trigger>
                <Portal>
                  <Popover.Positioner>
                    <Popover.Content>
                      <Popover.Arrow />
                      <Popover.Body>
                        <Text fontWeight="medium" mb={3}>Direction</Text>
                        <SegmentGroup.Root
                          size="sm"
                          defaultValue="TB"
                          value={direction}
                          onValueChange={(e) => setDirection(e.value as DiagramDirection)}
                        >
                          <SegmentGroup.Indicator />
                          <SegmentGroup.Items
                            items={[
                              { value: "TB", label: <LuArrowDown title="Top to Bottom" /> },
                              { value: "BT", label: <LuArrowUp title="Bottom to Top" /> },
                              { value: "LR", label: <LuArrowRight title="Left to Right" /> },
                              { value: "RL", label: <LuArrowLeft title="Right to Left" /> },
                            ]}
                          />
                        </SegmentGroup.Root>
                      </Popover.Body>
                    </Popover.Content>
                  </Popover.Positioner>
                </Portal>
              </Popover.Root>

              <Tabs.Root value={activeTab} onValueChange={(e) => setActiveTab(e.value)} fitted flex={1}>
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
            </Flex>

            <div style={{ position: "relative", flex: 1, minHeight: 0 }}>
              <div style={{ height: "100%", display: "flex", visibility: activeTab === "diagram" ? "visible" : "hidden", position: "absolute", top: 0, left: 0, right: 0, bottom: 0 }}>
                <DiagramView
                  diagramSvg={diagramSvg}
                  error={error}
                  isWasmLoaded={isWasmLoaded}
                  onFitToWidthReady={handleFitToWidthReady}
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
      </Flex>
    </Flex>
  );
}
