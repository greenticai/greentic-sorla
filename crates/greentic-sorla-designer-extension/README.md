# greentic-sorla-designer-extension

Deterministic Sorla Designer extension adapter.

The real Designer SDK/WIT is not vendored in this repository. This crate keeps
a narrow JSON tool boundary around `greentic-sorla-lib` so the implementation
can be swapped onto the SDK bindings when they are available.
