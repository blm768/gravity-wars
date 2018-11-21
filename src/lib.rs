#![feature(custom_attribute)]
#![feature(try_from)]

extern crate cgmath;
extern crate gltf;
extern crate js_sys;
extern crate rgb;
extern crate wasm_bindgen;
extern crate web_sys;

pub mod glue;
pub mod meshgen;
pub mod rendering;
pub mod state;
pub mod state_renderer;
