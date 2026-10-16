use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

/// 调用方分配的 JSON 输出缓冲区大小（与桥接层保持一致）。
pub const JSON_OUT_BUF_SIZE: usize = 65536;

/// 把 JSON 写进调用方提供的缓冲区，返回 0 成功、-1 失败。
fn write_json_out(json_out: *mut c_char, value: &Value) -> c_int {
    if json_out.is_null() {
        return -1;
    }
    let out_str = value.to_string();
    let bytes = out_str.as_bytes();
    if bytes.len() + 1 > JSON_OUT_BUF_SIZE {
        // 超长，写一个错误 JSON 而不是截断
        let fallback = json!({
            "code": 500900,
            "data": {},
            "msg": "json output exceeds buffer size"
        })
        .to_string();
        let fb = fallback.as_bytes();
        if fb.len() + 1 > JSON_OUT_BUF_SIZE {
            return -1;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(fb.as_ptr(), json_out as *mut u8, fb.len());
            *json_out.add(fb.len()) = 0;
        }
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());
        *json_out.add(bytes.len()) = 0;
    }
    0
}

/// 构造错误 JSON 并写进 json_out。
pub fn write_error_json(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(
        json_out,
        &json!({
            "code": code,
            "data": {},
            "msg": msg
        }),
    )
}

/// 构造成功 JSON 并写进 json_out。
pub fn write_success_json(json_out: *mut c_char, msg: &str, data: Value) -> c_int {
    write_json_out(
        json_out,
        &json!({
            "code": 200,
            "data": data,
            "msg": msg
        }),
    )
}

/// 从 C 字符串解析 JSON。
///
/// 这是内部辅助函数，只在导出函数内部使用。
/// 导出函数本身已是 `extern "C"` 且接收裸指针，安全边界在那里，
/// 因此这里显式允许解引用裸指针。
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn parse_json<T>(json_in: *const c_char) -> Result<T, String>
where
    T: DeserializeOwned,
{
    if json_in.is_null() {
        return Err("json_in is null".into());
    }
    let body = unsafe { CStr::from_ptr(json_in) }
        .to_str()
        .map_err(|e| e.to_string())?;
    serde_json::from_str(body).map_err(|e| e.to_string())
}
