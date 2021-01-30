/// Retrieves the real system time
pub fn real_time() -> u64 {
    js_sys::Date::now() as u64
}
