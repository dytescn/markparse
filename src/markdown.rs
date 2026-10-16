use crate::utils::cjson::{parse_json, write_error_json, write_success_json};
use serde::Deserialize;
use serde_json::json;
use std::ffi::c_char;
use std::os::raw::c_int;

// ============================================================
// 公共数据结构
// ============================================================

/// 调用方对解析/编译行为的可选项。
///
/// - `allow_dangerous_html`：是否允许原始 HTML 透传到输出（默认 false，安全）。
/// - `allow_dangerous_protocol`：是否允许 `javascript:` 等协议（默认 false）。
#[derive(Debug, Deserialize)]
struct MarkdownOptions {
    #[serde(default)]
    allow_dangerous_html: bool,
    #[serde(default)]
    allow_dangerous_protocol: bool,
}

impl MarkdownOptions {
    fn default_safe() -> Self {
        Self {
            allow_dangerous_html: false,
            allow_dangerous_protocol: false,
        }
    }

    fn to_options(&self) -> markdown::Options {
        let mut opts = markdown::Options::default();
        opts.compile.allow_dangerous_html = self.allow_dangerous_html;
        opts.compile.allow_dangerous_protocol = self.allow_dangerous_protocol;
        opts
    }
}

#[derive(Debug, Deserialize)]
struct ToHtmlReq {
    source: String,
    #[serde(default)]
    options: Option<MarkdownOptions>,
}

#[derive(Debug, Deserialize)]
struct ToMdastReq {
    source: String,
    #[serde(default)]
    options: Option<MarkdownOptions>,
}

// ============================================================
// 公共辅助
// ============================================================

/// 把 markdown-rs 的 mdast 节点序列化成 JSON 值。
///
/// 走 `serde_json::to_value`，因为 mdast 启用了 `serde` feature。
fn mdast_to_json(node: &markdown::mdast::Node) -> Result<serde_json::Value, String> {
    serde_json::to_value(node).map_err(|e| e.to_string())
}

// ============================================================
// FFI 导出接口：签名统一为 (json_in, json_out) -> c_int
// ============================================================

/// Markdown 转 HTML。
///
/// 输入 JSON:
/// ```json
/// {
///   "source": "# hi",
///   "options": { "allow_dangerous_html": false }
/// }
/// ```
/// `options` 可省略，默认全部关闭。
#[unsafe(no_mangle)]
pub extern "C" fn post_markdown_to_html(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: ToHtmlReq = match parse_json(json_in) {
        Ok(v) => v,
        Err(e) => return write_error_json(json_out, 400201, &e),
    };

    let opts = req.options.unwrap_or_else(MarkdownOptions::default_safe);

    match markdown::to_html_with_options(&req.source, &opts.to_options()) {
        Ok(html) => write_success_json(json_out, "success", json!({ "html": html })),
        Err(e) => write_error_json(json_out, 500201, &e.to_string()),
    }
}

/// Markdown 转 mdast（语法树）。
///
/// 输入 JSON 同 `post_markdown_to_html`。
/// 输出 `data.ast` 为 mdast 的 JSON 表示。
#[unsafe(no_mangle)]
pub extern "C" fn post_markdown_to_mdast(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let req: ToMdastReq = match parse_json(json_in) {
        Ok(v) => v,
        Err(e) => return write_error_json(json_out, 400202, &e),
    };

    let opts = req.options.unwrap_or_else(MarkdownOptions::default_safe);

    match markdown::to_mdast(&req.source, &opts.to_options().parse) {
        Ok(node) => match mdast_to_json(&node) {
            Ok(ast) => write_success_json(json_out, "success", json!({ "ast": ast })),
            Err(e) => write_error_json(json_out, 500202, &e),
        },
        Err(e) => write_error_json(json_out, 500203, &e.to_string()),
    }
}

