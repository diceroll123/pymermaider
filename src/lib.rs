pub mod checker;
pub mod class_diagram;
pub mod class_helpers;
pub mod class_type_detector;
pub mod mermaid_escape;
pub mod mermaid_renderer;
pub mod parameter_generator;
pub mod renderer;
pub mod type_analyzer;

pub use mermaid_renderer::RenderOptions;
pub use ruff_python_ast as ast;

use class_diagram::ClassDiagram;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PyMermaider {
    diagram: ClassDiagram,
    options: RenderOptions,
}

#[wasm_bindgen]
impl PyMermaider {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Initialize console_error_panic_hook for better error messages in the browser
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        Self {
            diagram: ClassDiagram::new(RenderOptions::default()),
            options: RenderOptions::default(),
        }
    }

    /// Set the diagram direction (TB, BT, LR, RL)
    #[wasm_bindgen(js_name = setDirection)]
    pub fn set_direction(&mut self, direction: &str) -> Result<(), JsValue> {
        self.options.direction = direction
            .parse()
            .map_err(|e: String| JsValue::from_str(&e))?;
        Ok(())
    }

    /// Get the current diagram direction
    #[wasm_bindgen(js_name = getDirection)]
    pub fn get_direction(&self) -> String {
        self.options.direction.to_string()
    }

    /// Set whether to hide private members (fields and methods with names starting with _) in the diagram. Off by default.
    #[wasm_bindgen(js_name = setHidePrivateMembers)]
    pub fn set_hide_private_members(&mut self, hide: bool) {
        self.options.hide_private_members = hide;
    }

    /// Process Python source code and return the Mermaid diagram as a string (or empty string if no diagram)
    #[wasm_bindgen(js_name = processPythonCode)]
    pub fn process_python_code(&mut self, source: &str) -> Result<String, JsValue> {
        // Reset the diagram for fresh processing (preserving options)
        self.diagram = ClassDiagram::new(self.options);

        // Add the source to the diagram
        self.diagram.add_source(source.to_string());

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
