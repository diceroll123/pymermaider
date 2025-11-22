import { Box } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";

interface ResizableDividerProps {
  isDragging: boolean;
  onMouseDown: () => void;
  onDoubleClick?: () => void;
  leftPosition: number; // Percentage position from left
}

export function ResizableDivider({
  isDragging,
  onMouseDown,
  onDoubleClick,
  leftPosition,
}: ResizableDividerProps) {
  const dividerBg = useColorModeValue("gray.300", "gray.600");
  const dividerHoverBg = useColorModeValue("blue.400", "blue.500");

  return (
    <Box
      w="4px"
      bg={isDragging ? dividerHoverBg : dividerBg}
      cursor="col-resize"
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
      transition="background 0.2s"
      _hover={{ bg: dividerHoverBg }}
      position="absolute"
      top={0}
      bottom={0}
      left={`calc(${leftPosition}% - 2px)`}
      zIndex={10}
    />
  );
}
