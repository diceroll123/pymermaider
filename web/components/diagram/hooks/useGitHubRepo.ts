"use client";

import { useState, useCallback, useRef } from "react";
import type { FileNode, RepoSource } from "../types";

interface GitHubContent {
  name: string;
  path: string;
  type: "file" | "dir";
  size?: number;
  download_url?: string;
  content?: string;
  encoding?: string;
}

interface UseGitHubRepoResult {
  files: FileNode[];
  repoSource: RepoSource | null;
  isLoading: boolean;
  error: string | null;
  loadRepo: (url: string) => Promise<void>;
  loadChildren: (path: string) => Promise<FileNode[]>;
  getFileContent: (path: string) => Promise<string | undefined>;
  clearFiles: () => void;
}

// Cache for file contents
const contentCache = new Map<string, string>();

// Parse GitHub URL to extract owner and repo
function parseGitHubUrl(url: string): { owner: string; repo: string } | null {
  // Handle various GitHub URL formats
  const patterns = [
    /github\.com\/([^/]+)\/([^/]+)/,
    /^([^/]+)\/([^/]+)$/,  // owner/repo format
  ];

  for (const pattern of patterns) {
    const match = url.match(pattern);
    if (match) {
      return {
        owner: match[1],
        repo: match[2].replace(/\.git$/, "").replace(/\/$/, ""),
      };
    }
  }

  return null;
}

// Skip common non-source directories
const SKIP_DIRS = new Set([
  "node_modules", ".git", "__pycache__", ".venv", "venv",
  ".tox", ".pytest_cache", "dist", "build", ".egg-info",
  ".github", ".vscode", ".idea", "htmlcov", ".mypy_cache",
]);

export function useGitHubRepo(): UseGitHubRepoResult {
  const [files, setFiles] = useState<FileNode[]>([]);
  const [repoSource, setRepoSource] = useState<RepoSource | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Store repo info for lazy loading
  const repoInfoRef = useRef<{ owner: string; repo: string } | null>(null);

  const fetchContents = useCallback(async (
    owner: string,
    repo: string,
    path: string = ""
  ): Promise<GitHubContent[]> => {
    const url = path
      ? `https://api.github.com/repos/${owner}/${repo}/contents/${path}`
      : `https://api.github.com/repos/${owner}/${repo}/contents`;

    const response = await fetch(url);

    if (!response.ok) {
      if (response.status === 404) {
        throw new Error("Repository not found. Make sure it's a public repository.");
      }
      if (response.status === 403) {
        throw new Error("Rate limit exceeded. Please try again later.");
      }
      throw new Error(`GitHub API error: ${response.status}`);
    }

    return response.json();
  }, []);

  const contentsToNodes = useCallback((contents: GitHubContent[]): FileNode[] => {
    const nodes: FileNode[] = [];

    for (const item of contents) {
      // Skip non-source directories
      if (item.type === "dir" && SKIP_DIRS.has(item.name)) {
        continue;
      }

      const isPython = item.name.endsWith(".py");

      if (item.type === "file") {
        // Only include Python files
        if (!isPython) continue;

        nodes.push({
          id: item.path,
          name: item.name,
          isPython: true,
        });
      } else if (item.type === "dir") {
        // Include all directories (we'll prune empty ones later or use lazy loading)
        nodes.push({
          id: item.path,
          name: item.name,
          children: [], // Empty, will be loaded lazily
          childrenCount: 1, // Indicate it has children (for lazy loading)
        });
      }
    }

    // Sort: directories first, then files, alphabetically
    nodes.sort((a, b) => {
      const aIsDir = !!a.children;
      const bIsDir = !!b.children;
      if (aIsDir !== bIsDir) return aIsDir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });

    return nodes;
  }, []);

  const loadRepo = useCallback(async (url: string) => {
    setIsLoading(true);
    setError(null);
    contentCache.clear();

    try {
      const parsed = parseGitHubUrl(url.trim());
      if (!parsed) {
        throw new Error("Invalid GitHub URL. Use format: github.com/owner/repo or owner/repo");
      }

      repoInfoRef.current = parsed;
      const { owner, repo } = parsed;

      const contents = await fetchContents(owner, repo);
      const nodes = contentsToNodes(contents);

      if (nodes.length === 0) {
        throw new Error("No Python files found in the repository root. Try expanding directories.");
      }

      setFiles(nodes);
      setRepoSource({
        type: "github",
        name: repo,
        owner,
        repo,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load repository");
      setFiles([]);
      setRepoSource(null);
      repoInfoRef.current = null;
    } finally {
      setIsLoading(false);
    }
  }, [fetchContents, contentsToNodes]);

  const loadChildren = useCallback(async (path: string): Promise<FileNode[]> => {
    if (!repoInfoRef.current) return [];

    const { owner, repo } = repoInfoRef.current;

    try {
      const contents = await fetchContents(owner, repo, path);
      return contentsToNodes(contents);
    } catch (err) {
      console.error("Failed to load children:", err);
      return [];
    }
  }, [fetchContents, contentsToNodes]);

  const getFileContent = useCallback(async (path: string): Promise<string | undefined> => {
    // Check cache first
    if (contentCache.has(path)) {
      return contentCache.get(path);
    }

    if (!repoInfoRef.current) return undefined;

    const { owner, repo } = repoInfoRef.current;

    try {
      const url = `https://api.github.com/repos/${owner}/${repo}/contents/${path}`;
      const response = await fetch(url);

      if (!response.ok) {
        throw new Error(`Failed to fetch file: ${response.status}`);
      }

      const data: GitHubContent = await response.json();

      if (data.content && data.encoding === "base64") {
        const content = atob(data.content);
        contentCache.set(path, content);
        return content;
      }

      // If content is too large, fetch via download_url
      if (data.download_url) {
        const rawResponse = await fetch(data.download_url);
        const content = await rawResponse.text();
        contentCache.set(path, content);
        return content;
      }

      return undefined;
    } catch (err) {
      console.error("Failed to fetch file content:", err);
      return undefined;
    }
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setRepoSource(null);
    setError(null);
    repoInfoRef.current = null;
    contentCache.clear();
  }, []);

  return {
    files,
    repoSource,
    isLoading,
    error,
    loadRepo,
    loadChildren,
    getFileContent,
    clearFiles,
  };
}
