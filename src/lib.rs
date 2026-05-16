#![deny(clippy::correctness)]
#![warn(clippy::suspicious, clippy::perf, clippy::style, clippy::complexity)]

pub(crate) mod analysis;
pub mod class_diagram;
pub mod render;

pub use render::mermaid_renderer::RenderOptions;
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
    #[must_use]
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
    ///
    /// # Errors
    /// Returns an error if `direction` is not one of: `TB`, `BT`, `LR`, `RL`.
    #[wasm_bindgen(js_name = setDirection)]
    pub fn set_direction(&mut self, direction: &str) -> Result<(), JsValue> {
        self.options.direction = direction
            .parse()
            .map_err(|e: String| JsValue::from_str(&e))?;
        Ok(())
    }

    /// Get the current diagram direction
    #[wasm_bindgen(js_name = getDirection)]
    #[must_use]
    pub fn get_direction(&self) -> String {
        self.options.direction.to_string()
    }

    /// Set whether to hide private members (fields and methods with names starting with _) in the diagram. Off by default.
    #[wasm_bindgen(js_name = setHidePrivateMembers)]
    #[allow(clippy::missing_const_for_fn)] // wasm_bindgen prohibits const fn on exported methods
    pub fn set_hide_private_members(&mut self, hide: bool) {
        self.options.hide_private_members = hide;
    }

    /// Process Python source code and return the Mermaid diagram as a string (or empty string if no diagram)
    ///
    /// # Errors
    /// Currently infallible; returns `Ok` in all cases. The `Result` type is retained for future compatibility.
    #[wasm_bindgen(js_name = processPythonCode)]
    pub fn process_python_code(&mut self, source: &str) -> Result<String, JsValue> {
        // Reset the diagram for fresh processing (preserving options)
        self.diagram = ClassDiagram::new(self.options);

        // Add the source to the diagram
        self.diagram.add_source(source);

        // Return the mermaid diagram as a string, or empty string if None
        Ok(self.diagram.render().unwrap_or_default())
    }

    /// Get the current diagram as a string (or empty string if no diagram)
    #[wasm_bindgen(js_name = getDiagram)]
    #[must_use]
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
