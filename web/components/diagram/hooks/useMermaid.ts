import { useState, useEffect, useCallback } from "react";
import mermaid from "mermaid";
import type { PyMermaiderClass } from "../types";

interface UseMermaidProps {
  wasmRef: React.RefObject<PyMermaiderClass | null>;
  pythonCode: string;
  isWasmLoaded: boolean;
  colorMode: string | undefined;
  themeMounted: boolean;
}

export function useMermaid({
  wasmRef,
  pythonCode,
  isWasmLoaded,
  colorMode,
  themeMounted,
}: UseMermaidProps) {
  const [mermaidCode, setMermaidCode] = useState("");
  const [diagramSvg, setDiagramSvg] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  // Initialize Mermaid with theme based on color mode
  useEffect(() => {
    if (!themeMounted || !colorMode) return;
    const theme = colorMode === "dark" ? "dark" : "default";
    mermaid.initialize({
      startOnLoad: false,
      theme,
      securityLevel: "loose",
    });
  }, [colorMode, themeMounted]);

  // Process Python code and generate diagram
  const generateDiagram = useCallback(async () => {
    if (!wasmRef.current) {
      setError("WASM module not loaded yet");
      return;
    }

    // Handle empty code gracefully
    if (!pythonCode || pythonCode.trim().length === 0) {
      setError(null);
      setDiagramSvg("");
      setMermaidCode("");
      return;
    }

    setIsProcessing(true);
    setError(null);
    setDiagramSvg(""); // Clear previous diagram

    try {
      // Try to process the Python code
      let diagram: string;
      try {
        diagram = wasmRef.current.processPythonCode(pythonCode);
      } catch {
        // Handle Python parsing/processing errors gracefully - show empty state instead of error
        setMermaidCode("");
        setDiagramSvg("");
        return;
      }

      setMermaidCode(diagram);

      // If diagram is empty, clear the SVG
      if (!diagram || diagram.trim().length === 0) {
        setDiagramSvg("");
        return;
      }

      // Strip markdown wrapper if present
      let cleanDiagram = diagram.trim();
      if (cleanDiagram.startsWith("```mermaid")) {
        cleanDiagram = cleanDiagram
          .replace(/^```mermaid\s*\n/, "")
          .replace(/\n```\s*$/, "");
      } else if (cleanDiagram.startsWith("```")) {
        cleanDiagram = cleanDiagram
          .replace(/^```\s*\n/, "")
          .replace(/\n```\s*$/, "");
      }

      // Render the diagram
      try {
        const { svg } = await mermaid.render(
          `mermaid-diagram-${Date.now()}`,
          cleanDiagram
        );

        // Fix SVG dimensions
        let fixedSvg = svg;
        const viewBoxMatch = svg.match(/viewBox="([^"]+)"/);
        if (viewBoxMatch) {
          const viewBoxParts = viewBoxMatch[1].split(/\s+/);
          if (viewBoxParts.length >= 4) {
            const width = viewBoxParts[2];
            const height = viewBoxParts[3];

            fixedSvg = fixedSvg.replace(/<svg([^>]*?)>/, (match, attributes) => {
              const newAttributes = attributes
                .replace(/\s+width="[^"]*"/g, "")
                .replace(/\s+height="[^"]*"/g, "")
                .replace(/\s+style="[^"]*"/g, "");
              return `<svg${newAttributes} width="${width}" height="${height}">`;
            });
          }
        }

        setDiagramSvg(fixedSvg);
      } catch {
        // Silently handle mermaid rendering errors - show empty diagram instead
        setDiagramSvg("");
      }
    } catch (err) {
      console.error("Unexpected error generating diagram:", err);
      setError(
        err instanceof Error ? err.message : "An unexpected error occurred"
      );
    } finally {
      setIsProcessing(false);
    }
  }, [pythonCode, wasmRef]);

  // Auto-generate on code change (debounced)
  useEffect(() => {
    if (!isWasmLoaded) return;

    // If code is empty, clear the diagram immediately
    if (!pythonCode || pythonCode.trim().length === 0) {
      setError(null);
      setDiagramSvg("");
      setMermaidCode("");
      return;
    }

    // Calculate line count for debounce
    const matches = pythonCode.match(/\n/g);
    const lineCount = matches ? matches.length + 1 : 1;
    const debounceDelay = lineCount > 500 ? 1000 : 400;

    const timer = setTimeout(() => {
      generateDiagram();
    }, debounceDelay);

    return () => clearTimeout(timer);
  }, [isWasmLoaded, pythonCode, generateDiagram]);

  // Re-render diagram when color mode changes (but NOT when diagramSvg changes)
  useEffect(() => {
    if (!themeMounted || !colorMode || !mermaidCode) return;

    const renderWithNewTheme = async () => {
      try {
        // If mermaid code is empty, don't try to render
        if (!mermaidCode || mermaidCode.trim().length === 0) {
          return;
        }

        let cleanDiagram = mermaidCode.trim();
        if (cleanDiagram.startsWith("```mermaid")) {
          cleanDiagram = cleanDiagram
            .replace(/^```mermaid\s*\n/, "")
            .replace(/\n```\s*$/, "");
        } else if (cleanDiagram.startsWith("```")) {
          cleanDiagram = cleanDiagram
            .replace(/^```\s*\n/, "")
            .replace(/\n```\s*$/, "");
        }

        const { svg } = await mermaid.render(
          `mermaid-diagram-${Date.now()}`,
          cleanDiagram
        );

        // Fix SVG dimensions
        let fixedSvg = svg;
        const viewBoxMatch = svg.match(/viewBox="([^"]+)"/);
        if (viewBoxMatch) {
          const viewBoxParts = viewBoxMatch[1].split(/\s+/);
          if (viewBoxParts.length >= 4) {
            const width = viewBoxParts[2];
            const height = viewBoxParts[3];

            fixedSvg = fixedSvg.replace(/<svg([^>]*?)>/, (match, attributes) => {
              const newAttributes = attributes
                .replace(/\s+width="[^"]*"/g, "")
                .replace(/\s+height="[^"]*"/g, "")
                .replace(/\s+style="[^"]*"/g, "");
              return `<svg${newAttributes} width="${width}" height="${height}">`;
            });
          }
        }

        setDiagramSvg(fixedSvg);
      } catch (err) {
        console.error("Error re-rendering diagram with new theme:", err);
      }
    };

    renderWithNewTheme();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [themeMounted, colorMode, mermaidCode]);

  return {
    mermaidCode,
    diagramSvg,
    error,
    isProcessing,
    generateDiagram,
  };
}
