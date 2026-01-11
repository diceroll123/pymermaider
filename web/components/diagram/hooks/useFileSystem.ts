"use client";

import { useState, useCallback } from "react";
import type { FileNode, RepoSource } from "../types";

interface FileSystemEntry {
  isFile: boolean;
  isDirectory: boolean;
  name: string;
  fullPath: string;
  file(callback: (file: File) => void): void;
  createReader(): DirectoryReader;
}

interface DirectoryReader {
  readEntries(
    successCallback: (entries: FileSystemEntry[]) => void,
    errorCallback?: (error: Error) => void
  ): void;
}

interface UseFileSystemResult {
  files: FileNode[];
  repoSource: RepoSource | null;
  isLoading: boolean;
  error: string | null;
  handleDrop: (e: React.DragEvent) => Promise<void>;
  handleFileInput: (files: FileList) => Promise<void>;
  getFileContent: (path: string) => string | undefined;
  clearFiles: () => void;
}

// Store file contents in memory
const fileContents = new Map<string, string>();

async function readFileContent(entry: FileSystemEntry): Promise<string> {
  return new Promise((resolve, reject) => {
    entry.file((file) => {
      const reader = new FileReader();
      reader.onload = () => resolve(reader.result as string);
      reader.onerror = () => reject(reader.error);
      reader.readAsText(file);
    });
  });
}

async function readAllEntries(reader: DirectoryReader): Promise<FileSystemEntry[]> {
  const entries: FileSystemEntry[] = [];

  const readBatch = (): Promise<FileSystemEntry[]> => {
    return new Promise((resolve, reject) => {
      reader.readEntries(resolve, reject);
    });
  };

  // Keep reading until no more entries
  let batch = await readBatch();
  while (batch.length > 0) {
    entries.push(...batch);
    batch = await readBatch();
  }

  return entries;
}

async function processEntry(
  entry: FileSystemEntry,
  parentPath: string = ""
): Promise<FileNode | null> {
  const fullPath = parentPath ? `${parentPath}/${entry.name}` : entry.name;
  const isPython = entry.name.endsWith(".py");

  if (entry.isFile) {
    // Store content for Python files
    if (isPython) {
      try {
        const content = await readFileContent(entry);
        fileContents.set(fullPath, content);
        return {
          id: fullPath,
          name: entry.name,
          content,
          isPython: true,
        };
      } catch {
        return null;
      }
    }
    // Include non-Python files as disabled items
    return {
      id: fullPath,
      name: entry.name,
      isPython: false,
    };
  }

  if (entry.isDirectory) {
    // Skip common non-source directories
    const skipDirs = ["node_modules", ".git", "__pycache__", ".venv", "venv", ".tox", ".pytest_cache", "dist", "build", ".egg-info"];
    if (skipDirs.some(skip => entry.name === skip || entry.name.endsWith(skip))) {
      return null;
    }

    const reader = entry.createReader();
    const entries = await readAllEntries(reader);

    const children: FileNode[] = [];
    for (const childEntry of entries) {
      const child = await processEntry(childEntry, fullPath);
      if (child) children.push(child);
    }

    // Only include directories that have Python files (directly or in subdirs)
    if (children.length === 0) return null;

    // Sort: directories first, then files, alphabetically
    children.sort((a, b) => {
      const aIsDir = !!a.children;
      const bIsDir = !!b.children;
      if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

    return {
      id: fullPath,
      name: entry.name,
      children,
    };
  }

  return null;
}

export function useFileSystem(): UseFileSystemResult {
  const [files, setFiles] = useState<FileNode[]>([]);
  const [repoSource, setRepoSource] = useState<RepoSource | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    setIsLoading(true);
    setError(null);
    fileContents.clear();

    try {
      const items = Array.from(e.dataTransfer.items);
      const entries: FileSystemEntry[] = [];

      for (const item of items) {
        if (item.kind === "file") {
          const entry = item.webkitGetAsEntry?.() as FileSystemEntry | null;
          if (entry) entries.push(entry);
        }
      }

      if (entries.length === 0) {
        throw new Error("No files or folders found in drop");
      }

      const rootNodes: FileNode[] = [];
      let rootName = "";

      for (const entry of entries) {
        const node = await processEntry(entry);
        if (node) {
          rootNodes.push(node);
          if (!rootName) rootName = entry.name;
        }
      }

      if (rootNodes.length === 0) {
        throw new Error("No Python files found in the dropped folder");
      }

      // Sort root nodes
      rootNodes.sort((a, b) => {
        const aIsDir = !!a.children;
        const bIsDir = !!b.children;
        if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });

      setFiles(rootNodes);
      setRepoSource({
        type: "local",
        name: rootName,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to read files");
      setFiles([]);
      setRepoSource(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const getFileContent = useCallback((path: string): string | undefined => {
    return fileContents.get(path);
  }, []);

  const handleFileInput = useCallback(async (fileList: FileList) => {
    setIsLoading(true);
    setError(null);
    fileContents.clear();

    try {
      const filesArray = Array.from(fileList);

      if (filesArray.length === 0) {
        throw new Error("No files selected");
      }

      // Build tree from file paths (webkitRelativePath gives us the path)
      const tree = new Map<string, FileNode>();
      let rootName = "";

      // Skip directories
      const skipDirs = ["node_modules", ".git", "__pycache__", ".venv", "venv", ".tox", ".pytest_cache", "dist", "build", ".egg-info"];

      for (const file of filesArray) {
        // webkitRelativePath is like "folder/subfolder/file.py"
        const relativePath = (file as File & { webkitRelativePath?: string }).webkitRelativePath || file.name;
        const parts = relativePath.split("/");

        // Get root folder name
        if (!rootName && parts.length > 0) {
          rootName = parts[0];
        }

        // Skip files in excluded directories
        if (parts.some(part => skipDirs.includes(part))) continue;

        const isPython = file.name.endsWith(".py");
        let content: string | undefined;

        // Read file content for Python files
        if (isPython) {
          content = await file.text();
          const fullPath = relativePath;
          fileContents.set(fullPath, content);
        }

        // Build tree structure
        let currentPath = "";
        for (let i = 0; i < parts.length; i++) {
          const part = parts[i];
          const parentPath = currentPath;
          currentPath = currentPath ? `${currentPath}/${part}` : part;

          if (i === parts.length - 1) {
            // This is a file
            const fileNode: FileNode = {
              id: currentPath,
              name: part,
              content,
              isPython,
            };
            tree.set(currentPath, fileNode);

            // Add to parent's children
            if (parentPath) {
              const parent = tree.get(parentPath);
              if (parent && parent.children) {
                if (!parent.children.find(c => c.id === currentPath)) {
                  parent.children.push(fileNode);
                }
              }
            }
          } else {
            // This is a directory
            if (!tree.has(currentPath)) {
              const dirNode: FileNode = {
                id: currentPath,
                name: part,
                children: [],
              };
              tree.set(currentPath, dirNode);

              // Add to parent's children
              if (parentPath) {
                const parent = tree.get(parentPath);
                if (parent && parent.children) {
                  if (!parent.children.find(c => c.id === currentPath)) {
                    parent.children.push(dirNode);
                  }
                }
              }
            }
          }
        }
      }

      // Get root nodes (nodes without a parent in the tree)
      const rootNodes: FileNode[] = [];
      tree.forEach((node, path) => {
        const parts = path.split("/");
        if (parts.length === 1) {
          rootNodes.push(node);
        }
      });

      if (rootNodes.length === 0) {
        throw new Error("No Python files found in the selected folder");
      }

      // Sort all children recursively
      const sortChildren = (nodes: FileNode[]) => {
        nodes.sort((a, b) => {
          const aIsDir = !!a.children;
          const bIsDir = !!b.children;
          if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
        nodes.forEach(node => {
          if (node.children) sortChildren(node.children);
        });
      };
      sortChildren(rootNodes);

      setFiles(rootNodes);
      setRepoSource({
        type: "local",
        name: rootName,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to read files");
      setFiles([]);
      setRepoSource(null);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setRepoSource(null);
    setError(null);
    fileContents.clear();
  }, []);

  return {
    files,
    repoSource,
    isLoading,
    error,
    handleDrop,
    handleFileInput,
    getFileContent,
    clearFiles,
  };
}
