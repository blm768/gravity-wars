#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate wasm_bindgen;

pub mod glue;
pub mod renderer;
pub mod state;
