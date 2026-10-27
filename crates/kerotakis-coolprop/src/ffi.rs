//! Hand-declared subset of CoolProp's C API (`include/CoolPropLib.h`, v6.8.0).
//!
//! Only two entry points are used. CoolProp keeps global state (fluid cache,
//! last error string), so every call goes through one process-wide mutex in
//! [`props_si`].

use std::ffi::{c_char, c_double, c_int, c_long, CStr, CString};
use std::sync::Mutex;

extern "C" {
    fn PropsSI(
        output: *const c_char,
        name1: *const c_char,
        prop1: c_double,
        name2: *const c_char,
        prop2: c_double,
        fluid: *const c_char,
    ) -> c_double;

    fn get_global_param_string(param: *const c_char, output: *mut c_char, n: c_int) -> c_long;
}

static LOCK: Mutex<()> = Mutex::new(());

fn global_string(guard: &std::sync::MutexGuard<'_, ()>, param: &str) -> String {
    let _ = guard;
    let param = CString::new(param).expect("static parameter name");
    let mut buf = vec![0 as c_char; 2048];
    // SAFETY: buffer length is passed; CoolProp NUL-terminates within it.
    let ok =
        unsafe { get_global_param_string(param.as_ptr(), buf.as_mut_ptr(), buf.len() as c_int) };
    if ok != 1 {
        return String::new();
    }
    // SAFETY: NUL-terminated by the callee, inside `buf`.
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

/// `PropsSI` with CoolProp's error string captured. CoolProp signals failure
/// by returning `HUGE_VAL` (or a non-finite value) and setting `errstring`.
pub(crate) fn props_si(
    output: &str,
    name1: &str,
    prop1: f64,
    name2: &str,
    prop2: f64,
    fluid: &str,
) -> Result<f64, String> {
    let c = |s: &str| CString::new(s).expect("static CoolProp key");
    let (o, n1, n2, f) = (c(output), c(name1), c(name2), c(fluid));
    let guard = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    // Clear any stale error before the call.
    let _ = global_string(&guard, "errstring");
    // SAFETY: all pointers are valid NUL-terminated strings for the call.
    let v = unsafe {
        PropsSI(
            o.as_ptr(),
            n1.as_ptr(),
            prop1,
            n2.as_ptr(),
            prop2,
            f.as_ptr(),
        )
    };
    if v.is_finite() && v.abs() < 1e300 {
        Ok(v)
    } else {
        let msg = global_string(&guard, "errstring");
        Err(if msg.is_empty() {
            "CoolProp returned a non-finite value".into()
        } else {
            msg
        })
    }
}

/// Upstream version and git revision as reported by the linked library.
pub(crate) fn version() -> (String, String) {
    let guard = LOCK.lock().unwrap_or_else(|p| p.into_inner());
    (
        global_string(&guard, "version"),
        global_string(&guard, "gitrevision"),
    )
}
