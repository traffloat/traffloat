//! Miscellaneous utilities.

use std::any::Any;
use std::cell::RefCell;

use derive_new::new;
use once_cell::unsync::OnceCell;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Retrieves the time rom the system monotonic clock up to microsecond precision
pub fn high_res_time() -> u64 {
    let window = web_sys::window().expect("Window uninitialized");
    let perf = window
        .performance()
        .expect("window.performance uninitialized");

    #[allow(clippy::cast_possible_truncation)]
    {
        (perf.now() * 1000.) as u64
    }
}

/// Runs the closure and measures the time.
pub fn measure(closure: impl FnOnce()) -> u64 {
    let start = high_res_time();
    closure();
    let end = high_res_time();
    end - start
}

#[wasm_bindgen(module = "/js/reified.js")]
extern "C" {
    unsafe fn reify_promise(value: JsValue) -> JsValue;
    unsafe fn reified_state(value: JsValue) -> u8;
    unsafe fn reified_value(value: JsValue) -> JsValue;
}

#[wasm_bindgen(module = "/js/error.js")]
extern "C" {
    unsafe fn handle_error(value: JsValue);
}

/// Passes the error to the JavaScript error handler.
pub fn error_handler(value: &str) {
    handle_error(value.into());
}

/// Wraps a possibly resolved promise.
pub struct ReifiedPromise<T> {
    unknown: RefCell<Option<(JsValue, Box<dyn Any>)>>,
    known: OnceCell<Result<T, ()>>,
}

impl<T> ReifiedPromise<T> {
    /// Wraps a new promise value.
    pub fn new(reified: JsValue, attachments: impl Any) -> Self {
        Self {
            unknown: RefCell::new(Some((reified, Box::new(attachments)))),
            known: OnceCell::new(),
        }
    }
}

impl<T: JsCast> ReifiedPromise<T> {
    /// Retrieves the result of the promise if it has been resolved.
    pub fn resolved_or_null(&self) -> Result<Option<&T>, js_sys::Error> {
        if let Some(known) = self.known.get() {
            return Ok(known.as_ref().ok());
        }

        let mut unknown = self.unknown.borrow_mut();
        let (reified, _) = unknown.as_mut().expect("known and unknown are both None!");

        let value = match reified_state(reified.clone()) {
            0 => return Ok(None),
            1 => {
                let value = reified_value(reified.clone());
                let value = value.dyn_into::<T>()?;
                Ok(value)
            }
            2 => {
                let err = reified_value(reified.clone());
                log::warn!("Promise failed with error: {:?}", err);
                Err(())
            }
            _ => unreachable!(),
        };

        if self.known.set(value).is_err() {
            unreachable!("self.known.get() was None");
        }
        Ok(self.known.get().expect("Just initialized").as_ref().ok())
    }
}

/// Linear interpolation from a to b, with ratio=0 as a and ratio=1 as b
pub fn lerp(a: f64, b: f64, ratio: f64) -> f64 {
    a * (1. - ratio) + b * ratio
}

#[wasm_bindgen(module = "/js/debugDiv.js")]
extern "C" {
    unsafe fn set_div_lines(div: JsValue, lines: &str);
}

/// Writer for debug lines in a div
#[derive(new)]
pub struct DebugWriter {
    div: web_sys::HtmlElement,
    #[new(default)]
    lines: String,
}

impl DebugWriter {
    /// Resets the writer buffer.
    pub fn reset(&mut self) {
        self.lines.clear();
    }

    /// Appends a line to the div.
    pub fn write(&mut self, line: impl AsRef<str>) {
        self.lines.push('\n');
        self.lines.push_str(line.as_ref());
    }

    /// Flushes the buffer to the div.
    pub fn flush(&self) {
        let div: &JsValue = &self.div;
        set_div_lines(div.clone(), &self.lines);
    }
}
