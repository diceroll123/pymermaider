"""Generate Mermaid class diagrams from Python source code."""

from importlib.metadata import PackageNotFoundError, version as _dist_version

from pymermaider.pymermaider import generate, to_markdown

__all__ = ["generate", "to_markdown"]

try:
    __version__ = _dist_version("pymermaider")
except PackageNotFoundError:  # pragma: no cover - e.g. running from source tree
    __version__ = "0.0.0"
