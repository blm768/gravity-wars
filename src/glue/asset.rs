use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::mem;
use std::rc::Rc;

use futures::{Async, Future};
use js_sys::{ArrayBuffer, Function, Promise, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::Response;

#[derive(Clone, Debug)]
pub enum FetchErrorType {
    NotFound,
    NetworkError,
    Interrupted,
    HttpError(u16),
    Other,
}

#[derive(Clone, Debug)]
pub struct FetchError {
    err_type: FetchErrorType,
    message: Option<String>,
}

impl FetchError {
    pub fn new(err_type: FetchErrorType, message: Option<String>) -> FetchError {
        FetchError { err_type, message }
    }

    pub fn from_http_status(status: u16) -> FetchError {
        if status == 404 {
            FetchError::new(FetchErrorType::NotFound, None)
        } else {
            FetchError::new(FetchErrorType::HttpError(status), None)
        }
    }

    pub fn not_found() -> FetchError {
        FetchError {
            err_type: FetchErrorType::NotFound,
            message: None,
        }
    }

    // TODO: remove?
    pub fn other() -> FetchError {
        FetchError {
            err_type: FetchErrorType::Other,
            message: None,
        }
    }
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

pub type FetchResult = Result<Vec<u8>, FetchError>;

#[derive(Debug)]
struct AssetLoaderData {
    pending: HashMap<Box<str>, Promise>,
    resolved: HashMap<Box<str>, FetchResult>,
}

impl AssetLoaderData {
    fn new() -> Rc<RefCell<AssetLoaderData>> {
        Rc::new(RefCell::new(AssetLoaderData {
            pending: HashMap::new(),
            resolved: HashMap::new(),
        }))
    }

    fn load(loader: &Rc<RefCell<AssetLoaderData>>, uri: &str) {
        let saved_uri: Box<str> = uri.into();
        let saved_loader = Rc::clone(&loader);
        let future = do_fetch(uri).then(move |result| {
            let mut borrowed = saved_loader.borrow_mut();
            borrowed.pending.remove(&saved_uri);
            borrowed.resolved.insert(saved_uri, result);
            Ok(JsValue::null())
        });
        let mut borrowed = loader.borrow_mut();
        borrowed
            .pending
            .insert(uri.into(), wasm_bindgen_futures::future_to_promise(future));
    }
}

pub struct AssetLoader {
    data: Rc<RefCell<AssetLoaderData>>,
}

impl AssetLoader {
    pub fn new() -> AssetLoader {
        AssetLoader {
            data: AssetLoaderData::new(),
        }
    }

    pub fn load(&self, uri: &str) {
        AssetLoaderData::load(&self.data, uri);
    }
}

impl Default for AssetLoader {
    fn default() -> Self {
        AssetLoader::new()
    }
}

impl Future for AssetLoader {
    type Item = AssetData;
    type Error = ();

    fn poll(&mut self) -> Result<Async<AssetData>, ()> {
        use crate::glue::log;
        log(&format!("Polling {:?}", &self.data.borrow()));
        let is_ready = {
            let borrowed = self.data.borrow();
            borrowed.pending.is_empty() && !borrowed.resolved.is_empty()
        };
        if is_ready {
            log("Ready");
            let data = mem::replace(&mut self.data.borrow_mut().resolved, HashMap::new());
            Ok(Async::Ready(AssetData(data)))
        } else {
            log("Not ready");
            Ok(Async::NotReady)
        }
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

fn error_to_message(error: JsValue) -> Option<String> {
    match error.dyn_into::<js_sys::Error>() {
        Ok(error) => Some(error.message().into()),
        Err(obj) => obj
            .dyn_into::<js_sys::Object>()
            .ok()
            .map(|o| o.to_string().into()),
    }
}

fn to_other_error(error: JsValue) -> FetchError {
    FetchError::new(FetchErrorType::Other, error_to_message(error))
}

fn do_fetch(uri: &str) -> impl Future<Item = Vec<u8>, Error = FetchError> {
    let promise = web_sys::window().unwrap().fetch_with_str(uri);
    JsFuture::from(promise)
        .map_err(|e| FetchError::new(FetchErrorType::NetworkError, error_to_message(e)))
        .and_then(move |response| {
            let response = response.dyn_into::<Response>().map_err(to_other_error)?;
            if response.ok() {
                Ok(response.array_buffer().map_err(to_other_error)?)
            } else {
                Err(FetchError::from_http_status(response.status()))
            }
        })
        .and_then(|promise| {
            JsFuture::from(promise)
                .map_err(|e| FetchError::new(FetchErrorType::Interrupted, error_to_message(e)))
        })
        .and_then(|obj| {
            let array = obj
                .dyn_into::<ArrayBuffer>()
                .map_err(to_other_error)
                .map(|buf| Uint8Array::new(&buf))?;
            let mut data = Vec::new();
            data.resize(array.length() as usize, 0);
            array.copy_to(&mut data[..]);
            Ok(data)
        })
}
