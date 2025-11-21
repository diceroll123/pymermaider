import { useState, useCallback, useRef } from "react";

export function useResizablePanel(initialWidth: number = 50) {
  const [leftPanelWidth, setLeftPanelWidth] = useState(initialWidth);
  const [isDragging, setIsDragging] = useState(false);

  const isDraggingRef = useRef(false);
  const mouseHandlersRef = useRef<{
    move: ((e: MouseEvent) => void) | null;
    up: ((e: MouseEvent) => void) | null;
  }>({ move: null, up: null });

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDraggingRef.current) return;

    const containerWidth = window.innerWidth;
    const newWidth = (e.clientX / containerWidth) * 100;

    // Constrain between 20% and 80%
    const constrainedWidth = Math.min(Math.max(newWidth, 20), 80);
    setLeftPanelWidth(constrainedWidth);
  }, []);

  const handleMouseUp = useCallback(() => {
    isDraggingRef.current = false;
    setIsDragging(false);
    if (mouseHandlersRef.current.move) {
      document.removeEventListener("mousemove", mouseHandlersRef.current.move);
    }
    if (mouseHandlersRef.current.up) {
      document.removeEventListener("mouseup", mouseHandlersRef.current.up);
    }
  }, []);

  const handleMouseDown = useCallback(() => {
    isDraggingRef.current = true;
    setIsDragging(true);
    mouseHandlersRef.current = { move: handleMouseMove, up: handleMouseUp };
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, [handleMouseMove, handleMouseUp]);

  const resetToCenter = useCallback(() => {
    setLeftPanelWidth(50);
  }, []);

  return {
    leftPanelWidth,
    isDragging,
    handleMouseDown,
    resetToCenter,
  };
}
