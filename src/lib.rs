//! # markparse
//!
//! Markdown 解析器，通过 C FFI 暴露两个接口：转 HTML、转 mdast。
//!
//! ## 快速开始
//!
//! ```no_run
//! use markparse::markdown::post_markdown_to_html;
//! use std::ffi::CString;
//!
//! let json_in = CString::new(
//!     serde_json::json!({ "source": "# hi" }).to_string()
//! ).unwrap();
//! let mut buf = vec![0u8; 65536];
//! let rc = post_markdown_to_html(json_in.as_ptr(), buf.as_mut_ptr() as *mut _);
//! assert_eq!(rc, 0);
//! ```
//!
//! ## 接口约定
//!
//! - 所有导出函数签名为 `(const char* json_in, char* json_out) -> int`
//! - `json_out` 缓冲区由调用方分配，至少 65536 字节
//! - 返回 `0` 表示 FFI 层成功，业务状态看 `json_out` 里的 `code` 字段
//! - 输入 JSON 由调用方拥有，函数不释放它
//!
//! ## 错误码
//!
//! - `400201`：转 HTML 时输入 JSON 非法
//! - `400202`：转 mdast 时输入 JSON 非法
//! - `500201`：转 HTML 内部错误
//! - `500202`：mdast 序列化为 JSON 失败
//! - `500203`：转 mdast 内部错误
//! - `500900`：输出超过 65536 字节缓冲
//!
//! ## 模块组织
//!
//! - [`markdown`]：两个 FFI 导出接口
//! - [`utils::cjson`]：JSON 编解码与缓冲区写入（供内部与测试使用）

pub mod markdown;
pub mod utils;
