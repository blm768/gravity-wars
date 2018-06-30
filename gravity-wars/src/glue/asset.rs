use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace=glue, js_name=fetchAsset)]
    fn fetch_asset(uri: &str, callback: &AssetLoadCallback);

    pub type Response;
    #[wasm_bindgen(js_namespace=glue, js_name=responseIsOK)]
    fn response_is_ok(response: &Response) -> bool;
}

pub type AssetLoadCallback = Closure<FnMut(Response, Vec<u8>)>;

struct AssetLoaderData {
    pending: HashMap<Box<str>, AssetLoadCallback>,
    resolved: HashMap<Box<str>, Result<Vec<u8>, ()>>,
    on_complete: Box<Fn(AssetData)>,
}

impl AssetLoaderData {
    fn new(callback: Box<Fn(AssetData)>) -> Rc<RefCell<AssetLoaderData>> {
        Rc::new(RefCell::new(AssetLoaderData {
            pending: HashMap::new(),
            resolved: HashMap::new(),
            on_complete: callback,
        }))
    }

    fn process_response(&mut self, uri: &str, response: Response, data: Vec<u8>) {
        self.pending.remove(uri);

        if response_is_ok(&response) {
            self.resolved.insert(uri.into(), Ok(data));
        } else {
            self.resolved.insert(uri.into(), Err(()));
        }

        if self.pending.len() == 0 {
            let data = AssetData(mem::replace(&mut self.resolved, HashMap::new()));
            (self.on_complete)(data);
        }
    }

    fn load(loader: Rc<RefCell<AssetLoaderData>>, uri: &str) {
        let saved_uri: Box<str> = uri.into();
        let saved_loader = Rc::clone(&loader);
        let callback = Closure::new(move |response: Response, data: Vec<u8>| {
            let mut borrowed = saved_loader.borrow_mut();
            // Without the clone, we seem to get a corrupted string. This is probably a Rust or wasm_bindgen bug.
            borrowed.process_response(&saved_uri.clone(), response, data);
        });
        let mut borrowed = loader.borrow_mut();
        fetch_asset(uri, &callback);
        borrowed.pending.insert(uri.into(), callback);
    }
}

#[wasm_bindgen]
pub struct AssetLoader {
    data: Rc<RefCell<AssetLoaderData>>,
}

impl AssetLoader {
    pub fn new<T: Fn(AssetData) + 'static>(callback: T) -> AssetLoader {
        AssetLoader {
            data: AssetLoaderData::new(Box::new(callback)),
        }
    }

    pub fn load(&self, uri: &str) {
        AssetLoaderData::load(Rc::clone(&self.data), uri);
    }
}

#[wasm_bindgen]
pub struct AssetData(pub HashMap<Box<str>, Result<Vec<u8>, ()>>);
