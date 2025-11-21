mod checker;
mod class_diagram;
mod class_helpers;
mod class_type_detector;
mod mermaid_escape;
mod mermaid_renderer;
mod parameter_generator;
mod renderer;
mod type_analyzer;

use class_diagram::ClassDiagram;
use ruff_python_ast as ast;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PyMermaider {
    diagram: ClassDiagram,
}

#[wasm_bindgen]
impl PyMermaider {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize console_error_panic_hook for better error messages in the browser
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self {
            diagram: ClassDiagram::new(),
        }
    }

    /// Process Python source code and return the Mermaid diagram as a string
    #[wasm_bindgen(js_name = processPythonCode)]
    pub fn process_python_code(&mut self, source: &str) -> Result<String, JsValue> {
        // Reset the diagram for fresh processing
        self.diagram = ClassDiagram::new();

        // Create a dummy path for the source
        let dummy_path = PathBuf::from("input.py");

        // Add the source to the diagram
        self.diagram.add_to_diagram(source.to_string(), &dummy_path);

        // Return the mermaid diagram as a string
        Ok(self.diagram.render())
    }

    /// Get the current diagram as a string
    #[wasm_bindgen(js_name = getDiagram)]
    pub fn get_diagram(&self) -> String {
        self.diagram.render()
    }
}

// Default implementation for non-WASM builds
impl Default for PyMermaider {
    fn default() -> Self {
        Self::new()
    }
}
