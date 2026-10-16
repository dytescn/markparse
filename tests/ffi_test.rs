//! 模拟 C 调用方，验证 ABI 契约。
//!
//! 手搓缓冲 + 裸指针，覆盖 C 侧真实调用场景。
//! 输入 JSON 一律走 `c_req()` 构造，避免 raw string 踩坑。

use markparse::markdown::{post_markdown_to_html, post_markdown_to_mdast};
use markparse::utils::cjson::JSON_OUT_BUF_SIZE;
use serde_json::Value;
use std::ffi::CStr;

/// 把 markdown 源包装成 NUL 结尾的 C 缓冲。
fn c_req(source: &str) -> Vec<u8> {
    let s = serde_json::json!({ "source": source }).to_string();
    let mut v = s.into_bytes();
    v.push(0);
    v
}

fn read_out(buf: &[u8]) -> Value {
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_str()
        .unwrap();
    serde_json::from_str(s).unwrap()
}

#[test]
fn abi_to_html_raw_pointers() {
    let input = c_req("# hi");
    let mut out = vec![0u8; JSON_OUT_BUF_SIZE];

    let rc = post_markdown_to_html(input.as_ptr() as *const _, out.as_mut_ptr() as *mut _);
    assert_eq!(rc, 0);

    let v = read_out(&out);
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["html"], "<h1>hi</h1>");
}

#[test]
fn abi_to_mdast_raw_pointers() {
    let input = c_req("# hi");
    let mut out = vec![0u8; JSON_OUT_BUF_SIZE];

    let rc = post_markdown_to_mdast(input.as_ptr() as *const _, out.as_mut_ptr() as *mut _);
    assert_eq!(rc, 0);

    let v = read_out(&out);
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["ast"]["type"], "root");
}

/// 出参缓冲区必须被 NUL 结尾 —— 不能依赖调用方预清零。
#[test]
fn abi_out_buffer_is_nul_terminated() {
    let input = c_req("hi");
    // 刻意填 0xFF，如果实现忘记写 NUL，CStr 会读到脏数据。
    let mut out = vec![0xFFu8; JSON_OUT_BUF_SIZE];

    let rc = post_markdown_to_html(input.as_ptr() as *const _, out.as_mut_ptr() as *mut _);
    assert_eq!(rc, 0);

    let v = read_out(&out);
    assert_eq!(v["code"], 200);
}

/// 用恰好等于 JSON_OUT_BUF_SIZE 的缓冲，验证不越界。
#[test]
fn abi_exact_size_buffer() {
    let input = c_req("ok");
    let mut out = vec![0u8; JSON_OUT_BUF_SIZE];

    let rc = post_markdown_to_html(input.as_ptr() as *const _, out.as_mut_ptr() as *mut _);
    assert_eq!(rc, 0);

    let v = read_out(&out);
    assert_eq!(v["code"], 200);
}
