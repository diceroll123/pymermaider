import { useRef, useEffect, useCallback } from "react";
import { Box, Text } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";
import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";
import { ZoomControls } from "./ZoomControls";
import { ErrorDisplay } from "./ErrorDisplay";
import { usePageVisibility } from "./hooks/usePageVisibility";

interface DiagramViewProps {
  diagramSvg: string;
  error: string | null;
  isWasmLoaded: boolean;
}

export function DiagramView({
  diagramSvg,
  error,
  isWasmLoaded,
}: DiagramViewProps) {
  const bgColor = useColorModeValue("white", "gray.800");
  const borderColor = useColorModeValue("gray.200", "gray.600");

  const isPageVisible = usePageVisibility();

  const diagramContainerRef = useRef<HTMLDivElement | null>(null);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const fitToWidthRef = useRef<(() => void) | null>(null);
  const originalSvgWidthRef = useRef<number | null>(null);
  const animationFrameRef = useRef<number | null>(null);
  const hasAutoFittedRef = useRef<string | null>(null); // Track which diagram has been auto-fitted

  // Extract SVG width accurately
  const extractSvgWidth = useCallback((svg: SVGSVGElement): number | null => {
    const viewBox = svg.getAttribute("viewBox");
    if (viewBox) {
      const parts = viewBox.split(/\s+/);
      if (parts.length >= 4) {
        const w = parseFloat(parts[2]);
        if (!isNaN(w) && w > 0) return w;
      }
    }

    if (svg.viewBox && svg.viewBox.baseVal) {
      const w = svg.viewBox.baseVal.width;
      if (w > 0) return w;
    }

    const widthAttr = svg.getAttribute("width");
    if (widthAttr && !widthAttr.includes("%")) {
      const w = parseFloat(widthAttr.replace("px", ""));
      if (!isNaN(w) && w > 0) return w;
    }

    if (
      svg.width &&
      svg.width.baseVal &&
      svg.width.baseVal.unitType !== SVGLength.SVG_LENGTHTYPE_PERCENTAGE
    ) {
      const w = svg.width.baseVal.value;
      if (w > 0) return w;
    }

    return null;
  }, []);

  // Store original SVG width and auto-fit when diagram loads (only once per diagram)
  useEffect(() => {
    originalSvgWidthRef.current = null;

    if (!diagramSvg || !diagramContainerRef.current || !isPageVisible) return;

    // Skip auto-fit if we've already fitted this diagram
    if (hasAutoFittedRef.current === diagramSvg) return;

    let retryCount = 0;
    const maxRetries = 30; // Increased retries for slower systems
    let timeoutId: NodeJS.Timeout | null = null;
    let fitRetryCount = 0;
    const maxFitRetries = 10;

    const measureAndFit = () => {
      // Cancel if page becomes hidden
      if (!isPageVisible) {
        if (animationFrameRef.current) {
          cancelAnimationFrame(animationFrameRef.current);
          animationFrameRef.current = null;
        }
        return;
      }

      const svg = diagramContainerRef.current?.querySelector("svg");
      if (!svg) {
        if (retryCount < maxRetries) {
          retryCount++;
          animationFrameRef.current = requestAnimationFrame(measureAndFit);
        }
        return;
      }

      const width = extractSvgWidth(svg as SVGSVGElement);

      if (width && width > 0) {
        originalSvgWidthRef.current = width;

        // Retry calling fitToWidth until it's available
        const tryFit = () => {
          if (fitToWidthRef.current && isPageVisible) {
            fitToWidthRef.current();
            // Mark this diagram as auto-fitted
            hasAutoFittedRef.current = diagramSvg;
          } else if (fitRetryCount < maxFitRetries) {
            fitRetryCount++;
            timeoutId = setTimeout(tryFit, 50);
          }
        };

        timeoutId = setTimeout(tryFit, 200);
      }
    };

    const timer = setTimeout(measureAndFit, 150);

    return () => {
      clearTimeout(timer);
      if (timeoutId) clearTimeout(timeoutId);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }
    };
  }, [diagramSvg, extractSvgWidth, isPageVisible]);

  return (
    <Box
      ref={containerRef}
      h="calc(100vh - 180px)"
      overflow="hidden"
      border="1px"
      borderColor={borderColor}
      borderRadius="md"
      bg={bgColor}
      position="relative"
    >
      {diagramSvg ? (
        <TransformWrapper
          initialScale={1}
          minScale={0.01}
          maxScale={100}
          centerOnInit={true}
          centerZoomedOut={true}
          doubleClick={{ disabled: true }}
          panning={{ velocityDisabled: true }}
        >
          {({ zoomIn, zoomOut, setTransform, centerView }) => {
            // Store fitToWidth function in ref for external access
            fitToWidthRef.current = () => {
              const container = containerRef.current;
              const svg = diagramContainerRef.current?.querySelector(
                "svg"
              ) as SVGSVGElement | null;

              if (!container || !svg) return;
              if (!setTransform || !centerView) return;

              // Get dimensions from viewBox for consistency
              const viewBox = svg.getAttribute("viewBox");
              if (!viewBox) return;
              const viewBoxParts = viewBox.split(/\s+/);
              if (viewBoxParts.length < 4) return;

              const svgWidth = parseFloat(viewBoxParts[2]);
              const svgHeight = parseFloat(viewBoxParts[3]);

              if (!svgWidth || svgWidth <= 0 || !svgHeight || svgHeight <= 0) return;

              const containerWidth = container.clientWidth;
              const containerHeight = container.clientHeight;
              if (containerWidth <= 0 || containerHeight <= 0) return;

              // Calculate scale to fit width with padding
              const scaleToFitWidth = (containerWidth * 0.95) / svgWidth;
              // Calculate scale to fit height with padding
              const scaleToFitHeight = (containerHeight * 0.95) / svgHeight;

              // Use the smaller scale to ensure diagram fits in both dimensions
              const targetScale = Math.min(scaleToFitWidth, scaleToFitHeight);

              // Set scale first, then center with no animation (0ms)
              setTransform(0, 0, targetScale, 0);
              centerView(targetScale, 0);
            };

            return (
              <>
                <ZoomControls
                  onZoomIn={() => zoomIn()}
                  onZoomOut={() => zoomOut()}
                  onFitToWidth={() => fitToWidthRef.current?.()}
                  onReset={() => fitToWidthRef.current?.()}
                />

                <TransformComponent
                  wrapperStyle={{
                    width: "100%",
                    height: "100%",
                  }}
                >
                  <Box
                    ref={diagramContainerRef}
                    dangerouslySetInnerHTML={{ __html: diagramSvg }}
                  />
                </TransformComponent>
              </>
            );
          }}
        </TransformWrapper>
      ) : (
        <Box
          display="flex"
          justifyContent="center"
          alignItems="center"
          h="100%"
        >
          {error ? (
            <ErrorDisplay error={error} variant="centered" />
          ) : (
            <Text color="gray.400">
              {isWasmLoaded
                ? "Enter Python code to see the diagram"
                : "Loading..."}
            </Text>
          )}
        </Box>
      )}
    </Box>
  );
}
