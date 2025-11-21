import { Box } from "@chakra-ui/react";
import { useColorModeValue } from "@/components/ui/color-mode";

interface ResizableDividerProps {
  isDragging: boolean;
  onMouseDown: () => void;
  onDoubleClick?: () => void;
}

export function ResizableDivider({
  isDragging,
  onMouseDown,
  onDoubleClick,
}: ResizableDividerProps) {
  const dividerBg = useColorModeValue("gray.300", "gray.600");
  const dividerHoverBg = useColorModeValue("blue.400", "blue.500");

  return (
    <Box
      w="4px"
      h="100%"
      bg={isDragging ? dividerHoverBg : dividerBg}
      cursor="col-resize"
      onMouseDown={onMouseDown}
      onDoubleClick={onDoubleClick}
      transition="background 0.2s"
      _hover={{ bg: dividerHoverBg }}
      position="relative"
      zIndex={10}
    />
  );
}
