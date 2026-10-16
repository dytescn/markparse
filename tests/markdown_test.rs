//! 针对两个 FFI 导出函数的测试。
//!
//! 所有输入都通过 `req()` 用 `serde_json::json!` 构造，
//! 避免手写 JSON 字符串被 Rust 字面量规则绊倒。

use markparse::markdown::{post_markdown_to_html, post_markdown_to_mdast};
use markparse::utils::cjson::JSON_OUT_BUF_SIZE;
use serde_json::Value;
use std::ffi::{CStr, CString};

/// 把 markdown 源包装成 FFI 入参 JSON 字符串。
fn req(source: &str) -> String {
    serde_json::json!({ "source": source }).to_string()
}

/// 同上，但带 options。
fn req_with_options(source: &str, opts: Value) -> String {
    serde_json::json!({ "source": source, "options": opts }).to_string()
}

fn call_html(input: &str) -> (i32, Value) {
    let c_in = CString::new(input).unwrap();
    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let rc = post_markdown_to_html(c_in.as_ptr(), buf.as_mut_ptr() as *mut _);
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_str()
        .unwrap()
        .to_owned();
    (rc, serde_json::from_str(&s).unwrap())
}

fn call_mdast(input: &str) -> (i32, Value) {
    let c_in = CString::new(input).unwrap();
    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let rc = post_markdown_to_mdast(c_in.as_ptr(), buf.as_mut_ptr() as *mut _);
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_str()
        .unwrap()
        .to_owned();
    (rc, serde_json::from_str(&s).unwrap())
}

// ---------- to_html：正常路径 ----------

#[test]
fn to_html_heading() {
    let (rc, v) = call_html(&req("# hi"));
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["html"], "<h1>hi</h1>");
}

#[test]
fn to_html_inline_formatting() {
    let (rc, v) = call_html(&req("**bold** and *italic*"));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn to_html_code_block() {
    let (rc, v) = call_html(&req("```rust\nfn main() {}\n```"));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(html.contains("<pre><code"));
    assert!(html.contains("fn main()"));
}

#[test]
fn to_html_empty_source() {
    let (rc, v) = call_html(&req(""));
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 200);
    assert_eq!(v["data"]["html"], "");
}

// ---------- to_html：options ----------

#[test]
fn to_html_default_dangerous_html_off() {
    let (rc, v) = call_html(&req("<script>alert(1)</script>"));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(!html.contains("<script>"), "html = {html}");
}

#[test]
fn to_html_allow_dangerous_html() {
    let (rc, v) = call_html(&req_with_options(
        "<script>alert(1)</script>",
        serde_json::json!({ "allow_dangerous_html": true }),
    ));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(html.contains("<script>alert(1)</script>"), "html = {html}");
}

#[test]
fn to_html_default_dangerous_protocol_off() {
    let (rc, v) = call_html(&req("[x](javascript:alert(1))"));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(!html.contains("javascript:"), "html = {html}");
}

#[test]
fn to_html_allow_dangerous_protocol() {
    let (rc, v) = call_html(&req_with_options(
        "[x](javascript:alert(1))",
        serde_json::json!({ "allow_dangerous_protocol": true }),
    ));
    assert_eq!(rc, 0);
    let html = v["data"]["html"].as_str().unwrap();
    assert!(html.contains("javascript:"), "html = {html}");
}

// ---------- to_html：错误路径 ----------

#[test]
fn to_html_bad_json_returns_400201() {
    let (rc, v) = call_html("not json");
    assert_eq!(rc, 0, "FFI 层仍成功，业务错误看 code");
    assert_eq!(v["code"], 400201);
}

#[test]
fn to_html_missing_source_returns_400201() {
    let (rc, v) = call_html("{}");
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 400201);
}

#[test]
fn to_html_null_in_returns_400201() {
    let mut buf = vec![0u8; JSON_OUT_BUF_SIZE];
    let rc = post_markdown_to_html(std::ptr::null(), buf.as_mut_ptr() as *mut _);
    assert_eq!(rc, 0);
    let s = unsafe { CStr::from_ptr(buf.as_ptr() as *const _) }
        .to_str()
        .unwrap();
    let v: Value = serde_json::from_str(s).unwrap();
    assert_eq!(v["code"], 400201);
}

// ---------- to_mdast ----------

#[test]
fn to_mdast_heading() {
    let (rc, v) = call_mdast(&req("# hi"));
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 200);
    let ast = &v["data"]["ast"];
    assert_eq!(ast["type"], "root");
    assert_eq!(ast["children"][0]["type"], "heading");
    assert_eq!(ast["children"][0]["depth"], 1);
}

#[test]
fn to_mdast_paragraph_with_text() {
    let (rc, v) = call_mdast(&req("hello world"));
    assert_eq!(rc, 0);
    let ast = &v["data"]["ast"];
    assert_eq!(ast["children"][0]["type"], "paragraph");
    assert_eq!(ast["children"][0]["children"][0]["value"], "hello world");
}

#[test]
fn to_mdast_bad_json_returns_400202() {
    let (rc, v) = call_mdast("{bad");
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 400202);
}

// ---------- 缓冲区边界 ----------

#[test]
fn to_mdast_oversize_returns_500900() {
    let mut src = String::new();
    for i in 0..5000 {
        src.push_str(&format!("paragraph number {i} with some filler text\n\n"));
    }
    let (rc, v) = call_mdast(&req(&src));
    assert_eq!(rc, 0);
    assert_eq!(v["code"], 500900, "expected oversize protection");
}
