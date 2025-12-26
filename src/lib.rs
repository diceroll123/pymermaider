pub mod checker;
pub mod class_diagram;
pub mod class_helpers;
pub mod class_type_detector;
pub mod mermaid_escape;
pub mod mermaid_renderer;
pub mod parameter_generator;
pub mod renderer;
pub mod type_analyzer;

pub use ruff_python_ast as ast;

use class_diagram::ClassDiagram;
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

    /// Process Python source code and return the Mermaid diagram as a string (or empty string if no diagram)
    #[wasm_bindgen(js_name = processPythonCode)]
    pub fn process_python_code(&mut self, source: &str) -> Result<String, JsValue> {
        // Reset the diagram for fresh processing
        self.diagram = ClassDiagram::new();

        // Create a dummy path for the source
        let dummy_path = PathBuf::from("input.py");

        // Add the source to the diagram
        self.diagram.add_to_diagram(source.to_string(), &dummy_path);

        // Return the mermaid diagram as a string, or empty string if None
        Ok(self.diagram.render().unwrap_or_default())
    }

    /// Get the current diagram as a string (or empty string if no diagram)
    #[wasm_bindgen(js_name = getDiagram)]
    pub fn get_diagram(&self) -> String {
        self.diagram.render().unwrap_or_default()
    }
}

// Default implementation for non-WASM builds
impl Default for PyMermaider {
    fn default() -> Self {
        Self::new()
    }
}
