"use client";

import { useState, useMemo, useEffect } from "react";
import { Box, Text, TreeView, createTreeCollection, Link } from "@chakra-ui/react";
import { LuExternalLink } from "react-icons/lu";
import { LuFile, LuFileCode, LuFolder, LuFolderOpen, LuLoaderCircle } from "react-icons/lu";
import type { FileNode, RepoSource } from "./types";

interface FileExplorerProps {
  files: FileNode[];
  repoSource: RepoSource | null;
  selectedFile: string | null;
  onFileSelect: (path: string) => void;
  onLoadChildren?: (path: string) => Promise<FileNode[]>;
}

function createCollection(files: FileNode[]) {
  return createTreeCollection<FileNode>({
    nodeToValue: (node) => node.id,
    nodeToString: (node) => node.name,
    rootNode: {
      id: "ROOT",
      name: "",
      children: files,
    },
  });
}

export function FileExplorer({
  files,
  repoSource,
  selectedFile,
  onFileSelect,
  onLoadChildren,
}: FileExplorerProps) {
  const initialCollection = useMemo(() => createCollection(files), [files]);
  const [collection, setCollection] = useState(initialCollection);

  // Reset collection when files change (e.g., new repo loaded)
  useEffect(() => {
    setCollection(createCollection(files));
  }, [files]);

  const handleSelectionChange = (details: { selectedValue: string[] }) => {
    const selected = details.selectedValue[0];
    if (selected) {
      // Only trigger file select for Python files (simple check by extension)
      if (selected.endsWith(".py")) {
        onFileSelect(selected);
      }
    }
  };

  const loadChildren = async (
    details: TreeView.LoadChildrenDetails<FileNode>
  ): Promise<FileNode[]> => {
    if (!onLoadChildren) return [];
    // valuePath contains full paths, use the last element (the node being expanded)
    const path = details.valuePath[details.valuePath.length - 1];
    return await onLoadChildren(path);
  };

  if (files.length === 0) {
    return null;
  }

  return (
    <Box h="100%" overflow="auto">
      {repoSource && (
        <Box
          px={3}
          py={2}
          borderBottomWidth="1px"
          borderColor="border.muted"
          bg="bg.subtle"
        >
          <Text fontSize="xs" fontWeight="semibold" color="fg.muted" textTransform="uppercase">
            {repoSource.type === "github" ? "GitHub" : "Local"}
          </Text>
          {repoSource.type === "github" ? (
            <Link
              href={`https://github.com/${repoSource.owner}/${repoSource.repo}`}
              target="_blank"
              rel="noopener noreferrer"
              fontSize="sm"
              fontWeight="medium"
              display="flex"
              alignItems="center"
              gap={1}
              _hover={{ textDecoration: "underline" }}
            >
              {repoSource.owner}/{repoSource.name}
              <LuExternalLink size={12} />
            </Link>
          ) : (
            <Text fontSize="sm" fontWeight="medium" truncate>
              {repoSource.name}
            </Text>
          )}
        </Box>
      )}

      <TreeView.Root
        collection={collection}
        size="sm"
        selectedValue={selectedFile ? [selectedFile] : []}
        onSelectionChange={handleSelectionChange}
        loadChildren={onLoadChildren ? loadChildren : undefined}
        onLoadChildrenComplete={(e) => setCollection(e.collection)}
      >
        <TreeView.Tree px={2} py={2}>
          <TreeView.Node
            indentGuide={<TreeView.BranchIndentGuide />}
            render={({ node, nodeState }) =>
              nodeState.isBranch ? (
                <TreeView.BranchControl>
                  <Box as="span" color="yellow.500" display="inline-flex">
                    {nodeState.loading ? (
                      <LuLoaderCircle style={{ animation: "spin 1s infinite" }} />
                    ) : nodeState.expanded ? (
                      <LuFolderOpen />
                    ) : (
                      <LuFolder />
                    )}
                  </Box>
                  <TreeView.BranchText>{node.name}</TreeView.BranchText>
                </TreeView.BranchControl>
              ) : (
                <TreeView.Item
                  opacity={node.isPython ? 1 : 0.5}
                  cursor={node.isPython ? "pointer" : "not-allowed"}
                  pointerEvents={node.isPython ? "auto" : "none"}
                >
                  <Box as="span" color={node.isPython ? "blue.400" : "fg.muted"} display="inline-flex">
                    {node.isPython ? <LuFileCode /> : <LuFile />}
                  </Box>
                  <TreeView.ItemText
                    fontWeight={node.isPython ? "medium" : "normal"}
                    color={node.isPython ? "fg" : "fg.muted"}
                  >
                    {node.name}
                  </TreeView.ItemText>
                </TreeView.Item>
              )
            }
          />
        </TreeView.Tree>
      </TreeView.Root>
    </Box>
  );
}
