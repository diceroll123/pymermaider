import { Box, Text } from "@chakra-ui/react";

interface ErrorDisplayProps {
  error: string;
  variant?: "inline" | "centered";
}

export function ErrorDisplay({ error, variant = "inline" }: ErrorDisplayProps) {
  if (variant === "centered") {
    return (
      <Box textAlign="center" maxW="md" p={6}>
        <Text fontSize="6xl" mb={4}>
          ⚠️
        </Text>
        <Text fontSize="lg" fontWeight="semibold" color="red.700" mb={2}>
          Unable to Generate Diagram
        </Text>
        <Text fontSize="sm" color="gray.600">
          {error}
        </Text>
      </Box>
    );
  }

  return (
    <Box
      p={4}
      bg="red.50"
      borderWidth="1px"
      borderColor="red.300"
      borderRadius="md"
    >
      <Text fontSize="sm" fontWeight="semibold" color="red.900" mb={1}>
        ⚠️ Error
      </Text>
      <Text fontSize="sm" color="red.800">
        {error}
      </Text>
    </Box>
  );
}
