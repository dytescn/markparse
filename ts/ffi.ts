const BASE_URL = "http://127.0.0.1:44944/markparse";

/**
 * 调用 FFI 符号。
 *
 * 注意：symbol 传原始函数名（如 "markdown_to_html"），
 * 桥接层会自动拼接 "post_" 前缀后再 dlsym。
 * 不要自己加 "post_"，否则会变成 "post_post_markdown_to_html"。
 */
export async function callFfi<T>(
  symbol: string,
  payload: Record<string, unknown>,
): Promise<T> {
  const response = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/libary",
      "FFI-Symbol": symbol,
    },
    body: JSON.stringify(payload),
  });

  if (!response.ok) {
    throw new Error(`HTTP 错误: ${response.status} - ${response.statusText}`);
  }

  const data = await response.json();
  if (data.code === 200) return data.data as T;
  throw new Error(`err info: ${data.msg} (code=${data.code})`);
}

/**
 * 无参 FFI 调用。
 *
 * 目前 markparse 没有无参接口，保留这个函数是为了和 sysfonts/ts/ffi.ts
 * 保持一致的结构，未来若有 list/version 之类接口可直接用。
 */
export async function callFfiNoArg<T>(symbol: string): Promise<T> {
  const response = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/libary",
      "FFI-Symbol": symbol,
    },
    body: "",
  });

  if (!response.ok) {
    throw new Error(`HTTP 错误: ${response.status} - ${response.statusText}`);
  }

  const data = await response.json();
  if (data.code === 200) return data.data as T;
  throw new Error(`err info: ${data.msg} (code=${data.code})`);
}

/**
 * 直接发原始 body，不做 JSON.stringify。
 *
 * 用于测非法 JSON 输入（触发 400201 / 400202）。
 * 也返回原始 { code, data, msg }，方便断言具体错误码。
 */
export async function callFfiRawBody(
  symbol: string,
  rawBody: string,
): Promise<{ code: number; data: unknown; msg: string }> {
  const response = await fetch(BASE_URL, {
    method: "POST",
    headers: {
      "Content-Type": "application/libary",
      "FFI-Symbol": symbol,
    },
    body: rawBody,
  });

  if (!response.ok) {
    throw new Error(`HTTP 错误: ${response.status} - ${response.statusText}`);
  }

  return await response.json();
}

// ============================================================
// 类型
// ============================================================

export interface MarkdownOptions {
  allow_dangerous_html?: boolean;
  allow_dangerous_protocol?: boolean;
}

export interface ToHtmlData {
  html: string;
}

export interface ToMdastData {
  ast: unknown;
}

export interface ToHtmlReq {
  source: string;
  options?: MarkdownOptions;
}

export interface ToMdastReq {
  source: string;
  options?: MarkdownOptions;
}
