use std::any::Any;
use std::cell::RefCell;
use std::ops::Deref;

use once_cell::unsync::OnceCell;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Retrieves the real system time
pub fn real_time() -> u64 {
    js_sys::Date::now() as u64
}

#[wasm_bindgen(module = "/js/reified.js")]
extern "C" {
    fn reify_promise(value: JsValue) -> JsValue;
    fn reified_state(value: JsValue) -> u8;
    fn reified_value(value: JsValue) -> JsValue;
}

pub struct ReifiedPromise<T> {
    unknown: RefCell<Option<(JsValue, Box<dyn Any>)>>,
    known: OnceCell<Result<T, ()>>,
}

impl<T> ReifiedPromise<T> {
    pub fn new(reified: JsValue, attachments: impl Any) -> Self {
        Self {
            unknown: RefCell::new(Some((reified, Box::new(attachments)))),
            known: OnceCell::new(),
        }
    }
}

impl<T: JsCast> ReifiedPromise<T> {
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
            },
            2 => {
                let err = reified_value(reified.clone());
                log::warn!("Promise failed with error: {:?}", err);
                Err(())
            },
            _ => unreachable!(),
        };

        if self.known.set(value).is_err() {
            unreachable!("self.known.get() was None");
        }
        Ok(self.known.get().expect("Just initialized").as_ref().ok())
    }
}
