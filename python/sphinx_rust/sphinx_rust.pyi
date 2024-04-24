from __future__ import annotations

__version__: str

def analyze_module(name: str, content: str) -> str:
    """Parse a module and return a high-level representation of it as a json string."""
