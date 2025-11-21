import { Box, VStack, Button } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";
import { MdZoomIn, MdZoomOut, MdFitScreen, MdRefresh } from "react-icons/md";

interface ZoomControlsProps {
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToWidth: () => void;
  onReset: () => void;
}

export function ZoomControls({
  onZoomIn,
  onZoomOut,
  onFitToWidth,
  onReset,
}: ZoomControlsProps) {
  const controlsBg = useColorModeValue("white", "gray.800");

  return (
    <Box
      position="absolute"
      top={4}
      right={4}
      zIndex={10}
      bg={controlsBg}
      borderRadius="md"
      boxShadow="lg"
      p={2}
    >
      <VStack gap={2}>
        <Button
          size="md"
          onClick={onZoomIn}
          colorScheme="blue"
          variant="solid"
          title="Zoom In"
          aria-label="Zoom In"
        >
          <MdZoomIn size={20} />
        </Button>
        <Button
          size="md"
          onClick={onZoomOut}
          colorScheme="blue"
          variant="solid"
          title="Zoom Out"
          aria-label="Zoom Out"
        >
          <MdZoomOut size={20} />
        </Button>
        <Button
          size="md"
          onClick={onFitToWidth}
          colorScheme="green"
          variant="solid"
          title="Fit to View"
          aria-label="Fit to View"
        >
          <MdFitScreen size={20} />
        </Button>
        <Button
          size="md"
          onClick={onReset}
          colorScheme="gray"
          variant="solid"
          title="Reset View"
          aria-label="Reset View"
        >
          <MdRefresh size={20} />
        </Button>
      </VStack>
    </Box>
  );
}
