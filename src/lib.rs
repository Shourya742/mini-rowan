/*
Custom implementation of rowan library - https://github.com/rust-analyzer/rowan
This structure should implement the following:
* full fidelity - whitespace and comments are part of the tree
* resilient & semi-structured - can parse incomplete code
* cheaply updatable - refactors and incremental parsing conveniently updatable
* immutable value type
* easy to navigate - from node to children, parent, siblings
*/

pub mod api;
pub mod arc;
pub mod ast;
pub mod cow_mut;
pub mod green;
pub mod red;
pub mod sll;
pub mod syntax;
pub mod syntax_text;
pub mod utility_types;
