import { useState, useEffect, useRef, useCallback } from "react";
import { createHighlighter, type Highlighter } from "shiki";
import { usePageVisibility } from "./usePageVisibility";

interface UseSyntaxHighlighterProps {
  code: string;
  colorMode: string | undefined;
  themeMounted: boolean;
}

export function useSyntaxHighlighter({
  code,
  colorMode,
  themeMounted,
}: UseSyntaxHighlighterProps) {
  const [highlighter, setHighlighter] = useState<Highlighter | null>(null);
  const [highlightedCode, setHighlightedCode] = useState<string>("");
  const [isHighlightingEnabled, setIsHighlightingEnabled] = useState(true);

  const isPageVisible = usePageVisibility();

  const highlightTimerRef = useRef<NodeJS.Timeout | null>(null);
  const pasteTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const prevCodeRef = useRef(code);

  // Calculate line count
  const lineCount = code.match(/\n/g)?.length ?? 0 + 1;

  // Initialize highlighter
  useEffect(() => {
    if (!themeMounted) return;
    let mounted = true;

    createHighlighter({
      themes: ["github-light", "github-dark"],
      langs: ["python"],
    }).then((h) => {
      if (mounted) setHighlighter(h);
    });

    return () => {
      mounted = false;
    };
  }, [themeMounted]);

  // Deferred highlighting for large files
  useEffect(() => {
    if (!themeMounted || !highlighter || !colorMode || !isHighlightingEnabled || !isPageVisible)
      return;

    // Clear previous timer
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
    }

    // Calculate delay based on file size
    const codeLength = code.length;
    let delay = 100;

    if (lineCount > 1000 || codeLength > 50000) {
      delay = 2000;
    } else if (lineCount > 500 || codeLength > 20000) {
      delay = 1000;
    } else if (lineCount > 200 || codeLength > 10000) {
      delay = 500;
    } else if (lineCount > 100 || codeLength > 5000) {
      delay = 300;
    }

    highlightTimerRef.current = setTimeout(() => {
      const performHighlighting = () => {
        const theme = colorMode === "dark" ? "github-dark" : "github-light";
        try {
          const html = highlighter.codeToHtml(code, {
            lang: "python",
            theme,
          });
          const match = html.match(/<code[^>]*>([\s\S]*?)<\/code>/);
          const highlighted = match ? match[1] : code;
          setHighlightedCode(highlighted);
        } catch {
          setHighlightedCode(code);
        }
      };

      // For large files, use requestIdleCallback
      if (codeLength > 5000 && typeof requestIdleCallback !== "undefined") {
        requestIdleCallback(performHighlighting, { timeout: 2000 });
      } else {
        performHighlighting();
      }
    }, delay);

    return () => {
      if (highlightTimerRef.current) {
        clearTimeout(highlightTimerRef.current);
      }
    };
  }, [themeMounted, code, highlighter, colorMode, lineCount, isHighlightingEnabled, isPageVisible]);

  // Cleanup paste timeout
  useEffect(() => {
    return () => {
      if (pasteTimeoutRef.current) {
        clearTimeout(pasteTimeoutRef.current);
      }
    };
  }, []);

  // Highlight function for editor
  const highlightCode = useCallback(
    (codeToHighlight: string) => {
      if (!codeToHighlight || codeToHighlight.trim().length === 0) {
        return "";
      }
      if (!isHighlightingEnabled || !highlighter) {
        return codeToHighlight;
      }

      if (highlightedCode && codeToHighlight === code) {
        return highlightedCode;
      }

      return codeToHighlight;
    },
    [highlighter, isHighlightingEnabled, highlightedCode, code]
  );

  // Handle code changes with paste optimization
  const handleCodeChange = useCallback((newCode: string) => {
    const lengthDiff = newCode.length - prevCodeRef.current.length;
    const isPaste = lengthDiff > 100;

    prevCodeRef.current = newCode;

    // For large pastes, temporarily disable highlighting
    if (isPaste) {
      setIsHighlightingEnabled(false);

      if (pasteTimeoutRef.current) {
        clearTimeout(pasteTimeoutRef.current);
      }

      const delay = lengthDiff > 5000 ? 500 : 200;
      pasteTimeoutRef.current = setTimeout(() => {
        setIsHighlightingEnabled(true);
      }, delay);
    }

    return newCode;
  }, []);

  return {
    highlightCode,
    handleCodeChange,
    isHighlightingEnabled,
    lineCount,
  };
}
