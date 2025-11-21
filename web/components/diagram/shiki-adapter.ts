import { createShikiAdapter } from "@chakra-ui/react";
import type { HighlighterGeneric } from "shiki";

// Create Shiki adapter for Chakra CodeBlock
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const shikiAdapter = createShikiAdapter<HighlighterGeneric<any, any>>({
  async load() {
    const { createHighlighter } = await import("shiki");
    return createHighlighter({
      langs: ["mermaid", "python"],
      themes: ["github-light", "github-dark"],
    });
  },
  theme: {
    light: "github-light",
    dark: "github-dark",
  },
});
