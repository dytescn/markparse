# markparse

[![Crates.io](https://img.shields.io/crates/v/markparse.svg)](https://crates.io/crates/markparse)
[![Documentation](https://docs.rs/markparse/badge.svg)](https://docs.rs/markparse)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

跨平台 Markdown 解析器，通过 C FFI 暴露两个接口：**转 HTML** 与 **转 mdast**。

底层基于 [`markdown`](https://crates.io/crates/markdown) crate，默认关闭一切危险特性（原始 HTML、危险协议），保证跨语言调用安全。

---

## 特性

- **纯函数 FFI**：无状态、无全局变量、可重入
- **统一签名**：所有导出函数为 `(const char* json_in, char* json_out) -> int`
- **统一返回**：`0` 表示 FFI 层成功，业务状态看 `json_out` 里的 `code` 字段
- **固定缓冲区**：调用方分配 65536 字节，超长会写 `500900` 错误 JSON 而不是截断
- **安全默认**：`allow_dangerous_html` 与 `allow_dangerous_protocol` 默认 `false`
- **无平台差异**：不像 `sysfonts` 需要 `cfg` 分发，一份代码跑全平台

---

## 快速开始

### 作为 Rust 库

```rust
use markparse::markdown::post_markdown_to_html;
use std::ffi::CString;

let json_in = CString::new(r#"{"source":"# hi"}"#).unwrap();
let mut buf = vec![0u8; 65536];
let rc = post_markdown_to_html(json_in.as_ptr(), buf.as_mut_ptr() as *mut _);
assert_eq!(rc, 0);
// buf → {"code":200,"data":{"html":"<h1>hi</h1>"},"msg":"success"}
```

### 作为 C 动态库

```c
#include <stdio.h>
#include <string.h>

extern int post_markdown_to_html(const char *json_in, char *json_out);

int main(void) {
    const char *in = "{\"source\":\"# hi\"}";
    char out[65536] = {0};

    int rc = post_markdown_to_html(in, out);
    if (rc != 0) {
        fprintf(stderr, "ffi failed\n");
        return 1;
    }
    printf("%s\n", out);
    // {"code":200,"data":{"html":"<h1>hi</h1>"},"msg":"success"}
    return 0;
}
```

编译时链接：

```bash
cc main.c -L target/release -lmarkparse -o demo
```

---

## 接口约定

所有导出函数遵循同一套协议：

| 项目 | 约定 |
| --- | --- |
| 函数签名 | `(const char *json_in, char *json_out) -> int` |
| `json_in` | UTF-8 JSON 字符串；由调用方拥有，函数不释放 |
| `json_out` | 调用方分配，至少 **65536 字节**；函数写入 NUL 结尾的 JSON |
| 返回值 | `0` 表示 FFI 层成功；`-1` 表示 `json_out` 为 null 或写入失败 |
| 业务状态 | 看 `json_out` 里的 `code` 字段 |
| 线程安全 | 是，函数无共享状态 |

### 成功响应

```json
{
  "code": 200,
  "data": { "...": "..." },
  "msg": "success"
}
```

### 错误响应

```json
{
  "code": 400201,
  "data": {},
  "msg": "expected value at line 1 column 1"
}
```

---

## 接口列表

### `post_markdown_to_html`

Markdown 转 HTML。

**输入**

```json
{
  "source": "# hi\n**bold**",
  "options": {
    "allow_dangerous_html": false,
    "allow_dangerous_protocol": false
  }
}
```

- `source`：必填，Markdown 源文本
- `options`：可选，缺省等价于两个开关都为 `false`

**输出**

```json
{
  "code": 200,
  "data": { "html": "<h1>hi</h1>\n<p><strong>bold</strong></p>" },
  "msg": "success"
}
```

### `post_markdown_to_mdast`

Markdown 转 mdast（语法树），输出为 JSON。

**输入**：同 `post_markdown_to_html`。

**输出**

```json
{
  "code": 200,
  "data": {
    "ast": {
      "type": "root",
      "children": [
        {
          "type": "heading",
          "depth": 1,
          "children": [{ "type": "text", "value": "hi" }]
        }
      ]
    }
  },
  "msg": "success"
}
```

mdast 的节点结构参见 [`markdown` crate 的 mdast 模块](https://docs.rs/markdown/latest/markdown/mdast/index.html)。

---

## 选项说明

| 字段 | 类型 | 默认 | 作用 |
| --- | --- | --- | --- |
| `allow_dangerous_html` | bool | `false` | 允许原始 HTML 透传到输出。开启后 `<script>` 等会原样保留，**跨语言调用时请谨慎** |
| `allow_dangerous_protocol` | bool | `false` | 允许 `javascript:`、`data:` 等协议出现在链接/图片里 |

> 默认关闭是为了防止跨语言调用方在不知情的情况下把不可信 Markdown 渲染成 HTML 后直接拼进页面，造成 XSS。

---

## 错误码

| 错误码 | 含义 | 触发条件 |
| --- | --- | --- |
| `400201` | 转 HTML 时输入 JSON 非法 | `json_in` 为 null、非 UTF-8、或不符合 `ToHtmlReq` |
| `400202` | 转 mdast 时输入 JSON 非法 | 同上 |
| `500201` | 转 HTML 内部错误 | `markdown` crate 编译阶段返回 `Err` |
| `500202` | mdast 序列化为 JSON 失败 | `serde_json::to_value` 返回 `Err` |
| `500203` | 转 mdast 内部错误 | `markdown` crate 解析阶段返回 `Err` |
| `500900` | 输出超过 65536 字节 | 结果 JSON 超过缓冲区，写回错误 JSON |

命名规则沿用 `sysfonts`：`4xxxxx` 参数类，`5xxxxx` 运行时类；`xx02xx` 段专属于 markparse。

---

## 项目结构

```
markparse/
├── Cargo.toml
├── src/
│   ├── lib.rs            # crate 文档与模块声明
│   ├── markdown.rs       # 两个 FFI 导出接口
│   └── utils/
│       ├── mod.rs
│       └── cjson.rs      # JSON 编解码与缓冲区写入
├── tests/                # FFI 冒烟测试
├── README.md
└── LICENSE
```

---

## 构建

```bash
# 作为 rlib 使用
cargo build --release

# 生成 cdylib（给 C / Python / Node 等调用）
cargo build --release
# 产物：
#   target/release/libmarkparse.so   (Linux)
#   target/release/libmarkparse.dylib (macOS)
#   target/release/markparse.dll     (Windows)
```

---

## 已知限制

- **固定 64KB 缓冲区**：大文档的 mdast 很容易超过，会命中 `500900`。后续计划增加 `post_markdown_to_mdast_alloc` 返回堆指针 + `free_json_string` 释放的版本。
- **未暴露 GFM / MDX**：目前只用 `markdown` crate 的默认特性。GFM 表格、任务列表等暂不支持。
- **不保留行尾信息**：mdast 的 `position` 字段默认未启用，如果调用方需要做源码映射，后续需增加开关。

---

## License

MIT，见 [LICENSE](LICENSE)。


