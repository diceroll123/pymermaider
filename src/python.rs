//! Python bindings (pyo3) exposing the diagram generator as a native module.

use crate::class_diagram::render_source_with_options;
use crate::render::renderer::DiagramDirection;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use ruff_python_ast::PythonVersion;

fn parse_direction(direction: &str) -> PyResult<DiagramDirection> {
    direction
        .parse()
        .map_err(|e: String| PyValueError::new_err(e))
}

fn parse_python_version(python_version: &str) -> PyResult<PythonVersion> {
    python_version
        .parse::<PythonVersion>()
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

fn render(
    source: &str,
    direction: &str,
    hide_private_members: bool,
    show_title: bool,
    title: Option<&str>,
    python_version: Option<&str>,
) -> PyResult<Option<String>> {
    let direction = parse_direction(direction)?;
    let python_version = match python_version {
        Some(version) => Some(parse_python_version(version)?),
        None => None,
    };
    Ok(render_source_with_options(
        source,
        direction,
        hide_private_members,
        show_title,
        title,
        python_version,
    ))
}

/// Generate a Mermaid class diagram from Python source code.
///
/// Returns the raw Mermaid `classDiagram` text, or an empty string if the
/// source contains no classes.
#[pyfunction]
#[pyo3(
    signature = (source, *, direction="TB", hide_private_members=false, show_title=true, title=None, python_version=None)
)]
fn generate(
    source: &str,
    direction: &str,
    hide_private_members: bool,
    show_title: bool,
    title: Option<&str>,
    python_version: Option<&str>,
) -> PyResult<String> {
    Ok(render(
        source,
        direction,
        hide_private_members,
        show_title,
        title,
        python_version,
    )?
    .unwrap_or_default())
}

/// Generate a Mermaid class diagram from Python source code, wrapped in a
/// fenced ` ```mermaid ` Markdown block. Returns an empty string if the source
/// contains no classes.
#[pyfunction]
#[pyo3(
    signature = (source, *, direction="TB", hide_private_members=false, show_title=true, title=None, python_version=None)
)]
fn to_markdown(
    source: &str,
    direction: &str,
    hide_private_members: bool,
    show_title: bool,
    title: Option<&str>,
    python_version: Option<&str>,
) -> PyResult<String> {
    let diagram = render(
        source,
        direction,
        hide_private_members,
        show_title,
        title,
        python_version,
    )?;
    Ok(match diagram {
        Some(diagram) => format!("```mermaid\n{}\n```\n", diagram.trim_end()),
        None => String::new(),
    })
}

/// The pymermaider Python module.
#[pymodule]
fn pymermaider(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(generate, module)?)?;
    module.add_function(wrap_pyfunction!(to_markdown, module)?)?;
    Ok(())
}
