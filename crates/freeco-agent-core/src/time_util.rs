/// Return the current time as milliseconds since the Unix epoch.
///
/// On native targets uses `std::time::SystemTime`.
/// On `wasm32` targets uses `js_sys::Date::now()`.
/// Returns `0` on any error (timestamps are informational).
pub(crate) fn now_ms() -> i64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
}
