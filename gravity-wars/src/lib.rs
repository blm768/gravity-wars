#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate cgmath;
extern crate wasm_bindgen;

pub mod glue;
pub mod mapgen;
pub mod rendering;
pub mod state;
