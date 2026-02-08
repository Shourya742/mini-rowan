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
mod arc;
pub mod ast;
mod cow_mut;
mod green;
pub mod red;
mod sll;
mod syntax_text;
mod utility_types;
