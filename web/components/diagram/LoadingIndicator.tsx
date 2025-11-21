import { Box, Text } from "@chakra-ui/react";

interface LoadingIndicatorProps {
  message?: string;
}

export function LoadingIndicator({
  message = "Loading WASM module...",
}: LoadingIndicatorProps) {
  return (
    <Box p={3} bg="yellow.100" borderRadius="md">
      <Text fontSize="sm" color="yellow.800">
        {message}
      </Text>
    </Box>
  );
}
