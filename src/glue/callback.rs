use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys;

pub struct AnimationFrameCallback {
    closure: Closure<Fn(f64)>,
    callback_handle: Rc<Cell<Option<i32>>>,
}

impl AnimationFrameCallback {
    pub fn new<F: Fn(f64) + 'static>(callback: F) -> AnimationFrameCallback {
        // We'll need to wrap the provided callback in a self-referential "loop" closure that runs the callback and then invokes requestAnimationFrame on itself.
        // To start, we'll need some shared storage to hold the self-referential closure. We can't fill it in yet because of the circular reference.
        let shared_loop: Rc<RefCell<Option<Box<Fn(f64)>>>> = Rc::new(RefCell::new(None));

        // Adapt a copy of shared_loop so it can be passed as a JS function to requestAnimationFrame.
        let weak_loop = Rc::downgrade(&shared_loop); // Prevents a reference loop.
        let loop_closure: Closure<Fn(f64)> = Closure::new(move |milliseconds: f64| {
            if let Some(strong) = weak_loop.upgrade() {
                if let Some(loop_func) = strong.borrow().as_ref() {
                    loop_func(milliseconds);
                }
            }
        });

        // Make shared storage to hold the most recent handle returned from requestAnimationFrame.
        let shared_handle: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));
        let handle_copy = Rc::clone(&shared_handle);

        // Now we actually define the self-referential closure.
        let callback_loop = move |milliseconds: f64| {
            callback(milliseconds);
            let next_handle = web_sys::window()
                .unwrap()
                .request_animation_frame(loop_closure.as_ref().unchecked_ref())
                .ok(); // NOTE: if requestAnimationFrame fails, the loop will stop silently.
            handle_copy.set(next_handle);
        };
        *shared_loop.borrow_mut() = Some(Box::new(callback_loop));

        let outer_closure: Closure<Fn(f64)> = Closure::new(move |milliseconds: f64| {
            if let Some(loop_func) = shared_loop.borrow().as_ref() {
                loop_func(milliseconds);
            }
        });

        AnimationFrameCallback {
            closure: outer_closure,
            callback_handle: shared_handle,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }
        let handle = web_sys::window()
            .unwrap()
            .request_animation_frame(self.closure.as_ref().unchecked_ref())
            .map_err(|_| String::from("Error in window.requestAnimationFrame()"))?;
        self.callback_handle.set(Some(handle));
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.callback_handle.get() {
            web_sys::window()
                .unwrap()
                .cancel_animation_frame(handle)
                .unwrap_or(());
            self.callback_handle.set(None);
        }
    }

    pub fn is_running(&self) -> bool {
        self.callback_handle.get().is_some()
    }
}

impl Drop for AnimationFrameCallback {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct IntervalCallback {
    closure: Closure<Fn()>,
    milliseconds: i32,
    callback_handle: Option<i32>,
}

impl IntervalCallback {
    pub fn new<F: Fn() + 'static>(callback: F, milliseconds: i32) -> IntervalCallback {
        let closure = Closure::new(callback);
        IntervalCallback {
            closure,
            milliseconds,
            callback_handle: None,
        }
    }

    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            return Ok(());
        }
        let handle = web_sys::window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments(
                self.closure.as_ref().unchecked_ref(),
                self.milliseconds,
                &Array::new(),
            )
            .map_err(|_| String::from("Error in window.setInterval()"))?;
        self.callback_handle = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.callback_handle {
            web_sys::window()
                .unwrap()
                .clear_interval_with_handle(handle);
            self.callback_handle = None;
        }
    }

    pub fn is_running(&self) -> bool {
        self.callback_handle.is_some()
    }
}

impl Drop for IntervalCallback {
    fn drop(&mut self) {
        self.stop();
    }
}
