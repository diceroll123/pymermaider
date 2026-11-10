import { useCallback, useMemo } from "react";

interface UseExportProps {
  mermaidCode: string;
  diagramSvg: string;
  colorMode?: string;
  baseName?: string | null;
}

/** Derive the export file stem from a path: "src/models/user.py" -> "user". */
function exportStem(baseName?: string | null): string {
  if (!baseName) return "diagram";
  const file = baseName.split("/").pop() ?? baseName;
  return file.replace(/\.(py|pyi)$/, "") || "diagram";
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function downloadText(text: string, filename: string, mimeType: string) {
  downloadBlob(new Blob([text], { type: mimeType }), filename);
}

/**
 * Rasterize an SVG string to a PNG Blob.
 * Renders at 2x the stated size for crisp output; when `background` is given
 * (dark mode) it fills the canvas first so the export matches what is on screen.
 */
async function svgToPngBlob(svg: string, background?: string): Promise<Blob> {
  const doc = new DOMParser().parseFromString(svg, "image/svg+xml");
  const svgEl = doc.documentElement;

  let width = parseFloat(svgEl.getAttribute("width") ?? "");
  let height = parseFloat(svgEl.getAttribute("height") ?? "");
  if (!Number.isFinite(width) || !Number.isFinite(height)) {
    const viewBox = (svgEl.getAttribute("viewBox") ?? "").split(/\s+/).map(Number);
    width = viewBox[2] ?? 0;
    height = viewBox[3] ?? 0;
  }
  if (width <= 0 || height <= 0) {
    throw new Error("Could not determine SVG dimensions");
  }

  const scale = 2;
  const canvas = document.createElement("canvas");
  canvas.width = Math.round(width * scale);
  canvas.height = Math.round(height * scale);

  const ctx = canvas.getContext("2d");
  if (!ctx) {
    throw new Error("Canvas 2D context unavailable");
  }

  if (background) {
    ctx.fillStyle = background;
    ctx.fillRect(0, 0, canvas.width, canvas.height);
  }

  const image = new Image();
  await new Promise<void>((resolve, reject) => {
    image.onload = () => resolve();
    image.onerror = () => reject(new Error("Failed to load SVG for PNG export"));
    image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  });
  ctx.drawImage(image, 0, 0, canvas.width, canvas.height);

  return await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => (blob ? resolve(blob) : reject(new Error("Failed to encode PNG"))),
      "image/png"
    );
  });
}

/** Export the current diagram as Mermaid code, Markdown, SVG or PNG. */
export function useExport({ mermaidCode, diagramSvg, colorMode, baseName }: UseExportProps) {
  const canExport = mermaidCode.trim().length > 0;
  const stem = useMemo(() => exportStem(baseName), [baseName]);

  /** Copy the raw Mermaid code to the clipboard. Resolves false on failure. */
  const copyMermaid = useCallback(async (): Promise<boolean> => {
    if (!canExport) return false;
    try {
      await navigator.clipboard.writeText(mermaidCode);
      return true;
    } catch {
      return false;
    }
  }, [canExport, mermaidCode]);

  const exportMmd = useCallback(() => {
    if (canExport) downloadText(mermaidCode, `${stem}.mmd`, "text/plain");
  }, [canExport, mermaidCode, stem]);

  const exportMarkdown = useCallback(() => {
    if (!canExport) return;
    downloadText("```mermaid\n" + mermaidCode.trim() + "\n```\n", `${stem}.md`, "text/markdown");
  }, [canExport, mermaidCode, stem]);

  const exportSvg = useCallback(() => {
    if (canExport && diagramSvg) downloadText(diagramSvg, `${stem}.svg`, "image/svg+xml");
  }, [canExport, diagramSvg, stem]);

  const exportPng = useCallback(async () => {
    if (!canExport || !diagramSvg) return;
    try {
      const blob = await svgToPngBlob(diagramSvg, colorMode === "dark" ? "#1d1d1d" : undefined);
      downloadBlob(blob, `${stem}.png`);
    } catch (err) {
      console.error("PNG export failed:", err);
    }
  }, [canExport, diagramSvg, colorMode, stem]);

  return { canExport, copyMermaid, exportMmd, exportMarkdown, exportSvg, exportPng };
}
