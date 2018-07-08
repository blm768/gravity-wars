use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::mem;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Response;
    #[wasm_bindgen(method, getter)]
    pub fn status(response: &Response) -> u32;

    #[wasm_bindgen(js_namespace=glue, js_name=fetchAsset)]
    fn fetch_asset(uri: &str, callback: &FetchCallback);
}

pub type FetchCallback = Closure<FnMut(WasmFetchResult)>;

#[derive(Clone, Debug)]
pub enum FetchErrorType {
    NotFound,
    NetworkError,
    Interrupted,
    HttpError(u32),
    Other,
}

#[derive(Clone, Debug)]
pub struct FetchError {
    err_type: FetchErrorType,
    message: Option<String>,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self.message {
            Some(ref msg) => Cow::Borrowed(&msg[..]),
            None => match self.err_type {
                FetchErrorType::NotFound => Cow::Borrowed("not found"),
                FetchErrorType::NetworkError => Cow::Borrowed("network error"),
                FetchErrorType::Interrupted => Cow::Borrowed("loading interrupted"),
                FetchErrorType::HttpError(code) => Cow::Owned(format!("HTTP error {}", code)),
                FetchErrorType::Other => Cow::Borrowed("unknown error"),
            },
        };
        write!(f, "Error while loading asset: {}", message)
    }
}

impl FetchError {
    pub fn new(err_type: FetchErrorType, message: Option<String>) -> FetchError {
        FetchError { err_type, message }
    }

    pub fn not_found() -> FetchError {
        FetchError {
            err_type: FetchErrorType::NotFound,
            message: None,
        }
    }
}

pub type FetchResult = Result<Vec<u8>, FetchError>;

#[wasm_bindgen]
pub struct WasmFetchResult(pub FetchResult);

#[wasm_bindgen]
impl WasmFetchResult {
    pub fn ok(data: Vec<u8>) -> WasmFetchResult {
        WasmFetchResult(Ok(data))
    }

    pub fn net_error(err: String) -> WasmFetchResult {
        WasmFetchResult(Err(FetchError::new(
            FetchErrorType::NetworkError,
            Some(err),
        )))
    }

    pub fn http_error(response: &Response) -> WasmFetchResult {
        let status_code = response.status();
        if status_code == 404 {
            WasmFetchResult(Err(FetchError::new(FetchErrorType::NotFound, None)))
        } else {
            WasmFetchResult(Err(FetchError::new(
                FetchErrorType::HttpError(response.status()),
                None,
            )))
        }
    }

    pub fn interrupted(err: String) -> WasmFetchResult {
        WasmFetchResult(Err(FetchError::new(FetchErrorType::Interrupted, Some(err))))
    }
}

struct AssetLoaderData {
    pending: HashMap<Box<str>, FetchCallback>,
    resolved: HashMap<Box<str>, FetchResult>,
    on_complete: Box<Fn(&AssetData)>,
}

impl AssetLoaderData {
    fn new(callback: Box<Fn(&AssetData)>) -> Rc<RefCell<AssetLoaderData>> {
        Rc::new(RefCell::new(AssetLoaderData {
            pending: HashMap::new(),
            resolved: HashMap::new(),
            on_complete: callback,
        }))
    }

    fn process_response(&mut self, uri: &str, result: FetchResult) {
        self.pending.remove(uri);

        self.resolved.insert(uri.into(), result);

        if self.pending.is_empty() {
            let data = AssetData(mem::replace(&mut self.resolved, HashMap::new()));
            (self.on_complete)(&data);
        }
    }

    fn load(loader: &Rc<RefCell<AssetLoaderData>>, uri: &str) {
        let saved_uri: Box<str> = uri.into();
        let saved_loader = Rc::clone(&loader);
        let callback = Closure::new(move |result: WasmFetchResult| {
            let mut borrowed = saved_loader.borrow_mut();
            // Without the clone, we seem to get a corrupted string. This is probably a Rust or wasm_bindgen bug.
            let WasmFetchResult(unwrapped) = result;
            borrowed.process_response(&saved_uri.clone(), unwrapped);
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
    pub fn new<T: Fn(&AssetData) + 'static>(callback: T) -> AssetLoader {
        AssetLoader {
            data: AssetLoaderData::new(Box::new(callback)),
        }
    }

    pub fn load(&self, uri: &str) {
        AssetLoaderData::load(&self.data, uri);
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct AssetData(HashMap<Box<str>, FetchResult>);

impl AssetData {
    pub fn get(&self, name: &str) -> Result<&[u8], FetchError> {
        let AssetData(ref data) = self;
        match data.get(name) {
            Some(result) => match result {
                Ok(ref data) => Ok(data),
                Err(ref err) => Err(err.clone()),
            },
            None => Err(FetchError::not_found()),
        }
    }
}
