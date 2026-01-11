"use client";

import { useState, useCallback, useRef } from "react";
import {
  Box,
  VStack,
  Text,
  Input,
  Button,
  HStack,
  Spinner,
  IconButton,
  List,
  Tooltip,
  Portal,
  Separator,
} from "@chakra-ui/react";
import { LuFolderInput, LuGithub, LuX, LuInfo } from "react-icons/lu";

interface RepoLoaderProps {
  isLoading: boolean;
  error: string | null;
  hasFiles: boolean;
  onDrop: (e: React.DragEvent) => Promise<void>;
  onFileInput: (files: FileList) => Promise<void>;
  onLoadGitHub: (url: string) => Promise<void>;
  onClear: () => void;
}

export function RepoLoader({
  isLoading,
  error,
  hasFiles,
  onDrop,
  onFileInput,
  onLoadGitHub,
  onClear,
}: RepoLoaderProps) {
  const [isDragOver, setIsDragOver] = useState(false);
  const [githubUrl, setGithubUrl] = useState("");
  const [showGitHubInput, setShowGitHubInput] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragOver(false);

      // Check if a URL was dropped (e.g., dragging a link from browser)
      const url = e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text/plain");
      if (url && url.includes("github.com")) {
        await onLoadGitHub(url.trim());
        return;
      }

      // Otherwise, handle as file drop
      await onDrop(e);
    },
    [onDrop, onLoadGitHub]
  );

  const handleGitHubSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (githubUrl.trim()) {
        await onLoadGitHub(githubUrl.trim());
        setGithubUrl("");
        setShowGitHubInput(false);
      }
    },
    [githubUrl, onLoadGitHub]
  );

  const handleDropZoneClick = useCallback(() => {
    if (!isLoading && fileInputRef.current) {
      fileInputRef.current.click();
    }
  }, [isLoading]);

  const handleFileInputChange = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (files && files.length > 0) {
        await onFileInput(files);
      }
      // Reset input so same folder can be selected again
      e.target.value = "";
    },
    [onFileInput]
  );

  // Show clear button when files are loaded
  if (hasFiles) {
    return (
      <Box px={3} py={2} borderBottomWidth="1px" borderColor="border.muted">
        <Button
          size="xs"
          variant="ghost"
          colorPalette="red"
          onClick={onClear}
          w="100%"
        >
          <LuX />
          Close Project
        </Button>
      </Box>
    );
  }

  return (
    <VStack h="100%" p={4} gap={4} align="stretch">
      {/* Hidden File Input */}
      <input
        ref={fileInputRef}
        type="file"
        // @ts-expect-error webkitdirectory is a non-standard attribute
        webkitdirectory=""
        directory=""
        multiple
        style={{ display: "none" }}
        onChange={handleFileInputChange}
      />

      {/* Drop Zone */}
      <Box
        border="2px dashed"
        borderColor={isDragOver ? "blue.400" : "border.muted"}
        borderRadius="lg"
        p={6}
        textAlign="center"
        bg={isDragOver ? "blue.50" : "bg.subtle"}
        _dark={{
          bg: isDragOver ? "blue.900/20" : "bg.subtle",
        }}
        transition="all 0.2s"
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={handleDropZoneClick}
        cursor="pointer"
        _hover={{
          borderColor: isLoading ? undefined : "blue.400",
        }}
      >
        {isLoading ? (
          <VStack gap={2}>
            <Spinner size="lg" color="blue.500" />
            <Text fontSize="sm" color="fg.muted">
              Loading files...
            </Text>
          </VStack>
        ) : (
          <VStack gap={2}>
            <LuFolderInput size={32} style={{ opacity: 0.5 }} />
            <Text fontSize="sm" fontWeight="medium">
              Drop folder or GitHub link
            </Text>
            <Text fontSize="xs" color="fg.muted" fontStyle="italic">
              Or click to select a folder
            </Text>
            <Separator w="full" />
            <Text fontSize="xs" color="fg.muted" fontStyle="italic">
              Private by design, nothing is uploaded anywhere.
            </Text>
          </VStack>
        )}
      </Box>

      {/* Divider */}
      <HStack gap={2}>
        <Box flex={1} h="1px" bg="border.muted" />
        <Text fontSize="xs" color="fg.muted">
          or
        </Text>
        <Box flex={1} h="1px" bg="border.muted" />
      </HStack>

      {/* GitHub Input */}
      {showGitHubInput ? (
        <form onSubmit={handleGitHubSubmit}>
          <VStack gap={2} align="stretch">
            <Input
              size="sm"
              placeholder="github.com/owner/repo"
              value={githubUrl}
              onChange={(e) => setGithubUrl(e.target.value)}
              autoFocus
            />
            <HStack gap={2}>
              <Button
                size="sm"
                variant="outline"
                flex={1}
                onClick={() => {
                  setShowGitHubInput(false);
                  setGithubUrl("");
                }}
              >
                Cancel
              </Button>
              <Button
                size="sm"
                colorPalette="blue"
                flex={1}
                type="submit"
                disabled={!githubUrl.trim() || isLoading}
              >
                {isLoading ? <Spinner size="sm" /> : "Load"}
              </Button>
            </HStack>
          </VStack>
        </form>
      ) : (
        <HStack gap={2}>
          <Button
            size="sm"
            variant="outline"
            onClick={() => setShowGitHubInput(true)}
            disabled={isLoading}
            flex={1}
          >
            <LuGithub />
            Load from GitHub
          </Button>
          <Tooltip.Root openDelay={0} closeDelay={100} positioning={{ placement: "top" }}>
            <Tooltip.Trigger asChild>
              <IconButton
                aria-label="GitHub API info"
                size="sm"
                variant="ghost"
              >
                <LuInfo />
              </IconButton>
            </Tooltip.Trigger>
            <Portal>
              <Tooltip.Positioner>
                <Tooltip.Content maxW="220px" px={3} py={2}>
                  <Tooltip.Arrow>
                    <Tooltip.ArrowTip />
                  </Tooltip.Arrow>
                  <Text fontSize="xs" fontWeight="semibold" mb={1}>
                    GitHub API Limitations
                  </Text>
                  <List.Root as="ul" fontSize="xs" gap={0.5} ps={4} opacity={0.85}>
                    <List.Item>Public repositories only</List.Item>
                    <List.Item>
                        60 requests/hour without auth
                        <List.Root ps={2}>
                            <List.Item><Text fontStyle="italic">We do not support auth</Text></List.Item>
                        </List.Root>
                    </List.Item>
                    <List.Item>Folders load on-demand</List.Item>
                  </List.Root>
                </Tooltip.Content>
              </Tooltip.Positioner>
            </Portal>
          </Tooltip.Root>
        </HStack>
      )}

      {/* Error Display */}
      {error && (
        <Box
          p={3}
          borderRadius="md"
          bg="red.50"
          _dark={{ bg: "red.900/20" }}
          borderWidth="1px"
          borderColor="red.200"
          _darkBorderColor="red.800"
        >
          <Text fontSize="xs" color="red.600" _dark={{ color: "red.300" }}>
            {error}
          </Text>
        </Box>
      )}
    </VStack>
  );
}
