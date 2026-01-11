import { useState, useEffect, useRef } from "react";
import type { PyMermaiderClass } from "../types";

export function useWasm() {
  const [isWasmLoaded, setIsWasmLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wasmRef = useRef<PyMermaiderClass | null>(null);

  useEffect(() => {
    let mounted = true;

    async function loadWasm() {
      try {
        console.log("Loading WASM module...");
        const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "";
        const jsUrl = `${basePath}/wasm/pymermaider_wasm.js`;
        const wasmUrl = `${basePath}/wasm/pymermaider_wasm_bg.wasm`;

        console.log("WASM JS URL:", jsUrl);
        console.log("WASM binary URL:", wasmUrl);

        // Dynamically load the WASM module
        const response = await fetch(jsUrl);
        if (!response.ok) {
          throw new Error(
            `Failed to fetch WASM JS: ${response.status} ${response.statusText}`
          );
        }

        const jsCode = await response.text();

        // Create a blob URL for the JS code
        const blob = new Blob([jsCode], { type: "application/javascript" });
        const blobUrl = URL.createObjectURL(blob);

        // Import the module
        const wasmModule = await import(/* webpackIgnore: true */ blobUrl);

        // Initialize the WASM
        await wasmModule.default(wasmUrl);

        if (!mounted) return;

        // Get the PyMermaider class
        const PyMermaider = wasmModule.PyMermaider;
        if (!PyMermaider) {
          throw new Error("PyMermaider class not found in WASM module");
        }

        console.log("WASM module loaded successfully");
        wasmRef.current = new PyMermaider();
        setIsWasmLoaded(true);
        setError(null);

        // Clean up blob URL
        URL.revokeObjectURL(blobUrl);
      } catch (err) {
        console.error("Failed to load WASM module:", err);
        if (mounted) {
          setError(
            `Failed to load WASM: ${err instanceof Error ? err.message : String(err)}`
          );
        }
      }
    }

    loadWasm();

    return () => {
      mounted = false;
    };
  }, []);

  return { wasmRef, isWasmLoaded, error };
}
