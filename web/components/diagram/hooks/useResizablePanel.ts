import { useState, useCallback, useRef, RefObject } from "react";

export function useResizablePanel(
  initialWidth: number = 50,
  containerRef?: RefObject<HTMLDivElement | null>
) {
  const [leftPanelWidth, setLeftPanelWidth] = useState(initialWidth);
  const [isDragging, setIsDragging] = useState(false);

  const isDraggingRef = useRef(false);
  const mouseHandlersRef = useRef<{
    move: ((e: MouseEvent) => void) | null;
    up: ((e: MouseEvent) => void) | null;
  }>({ move: null, up: null });

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDraggingRef.current) return;

    // Use container bounds if ref provided, otherwise fall back to window
    const container = containerRef?.current;
    let containerLeft = 0;
    let containerWidth = window.innerWidth;

    if (container) {
      const rect = container.getBoundingClientRect();
      containerLeft = rect.left;
      containerWidth = rect.width;
    }

    const relativeX = e.clientX - containerLeft;
    const newWidth = (relativeX / containerWidth) * 100;

    // Constrain between 20% and 80%
    const constrainedWidth = Math.min(Math.max(newWidth, 20), 80);
    setLeftPanelWidth(constrainedWidth);
  }, [containerRef]);

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
