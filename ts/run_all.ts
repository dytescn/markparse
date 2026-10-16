import {
  callFfi,
  callFfiRawBody,
  type ToHtmlData,
  type ToMdastData,
} from "./ffi.ts";

// ============================================================
// 全量冒烟：按顺序调一遍两个接口，各自覆盖正常路径和错误路径。
// 适合 CI 或本地快速回归。
// ============================================================

let failed = 0;

function ok(name: string) {
  console.log(`  ok   ${name}`);
}

function fail(name: string, reason: string) {
  failed += 1;
  console.error(`  FAIL ${name}: ${reason}`);
}

async function caseHtml(
  name: string,
  source: string,
  expect: (html: string) => boolean,
) {
  try {
    const data = await callFfi<ToHtmlData>("markdown_to_html", { source });
    if (!expect(data.html)) {
      fail(name, `html 不符合预期: ${JSON.stringify(data.html)}`);
    } else {
      ok(name);
    }
  } catch (e) {
    fail(name, (e as Error).message);
  }
}

async function caseMdast(
  name: string,
  source: string,
  expect: (ast: any) => boolean,
) {
  try {
    const data = await callFfi<ToMdastData>("markdown_to_mdast", { source });
    if (!expect(data.ast)) {
      fail(name, `ast 不符合预期`);
    } else {
      ok(name);
    }
  } catch (e) {
    fail(name, (e as Error).message);
  }
}

/** 发原始 body，断言业务错误码。 */
async function caseRawCode(
  name: string,
  symbol: string,
  rawBody: string,
  expectedCode: number,
) {
  try {
    const r = await callFfiRawBody(symbol, rawBody);
    if (r.code !== expectedCode) {
      fail(name, `期望 code=${expectedCode}，实际 code=${r.code}，msg=${r.msg}`);
    } else {
      ok(name);
    }
  } catch (e) {
    fail(name, (e as Error).message);
  }
}

console.log("markdown_to_html:");
await caseHtml("heading", "# hi", (h) => h === "<h1>hi</h1>");
await caseHtml("inline", "**b** *i*", (h) =>
  h.includes("<strong>b</strong>") && h.includes("<em>i</em>"));
await caseHtml("code block", "```\nlet x = 1;\n```", (h) =>
  h.includes("<pre><code") && h.includes("let x = 1;"));
await caseHtml("empty", "", (h) => h === "");
await caseHtml("escape by default", "<script>alert(1)</script>", (h) =>
  !h.includes("<script>"));

console.log("markdown_to_mdast:");
await caseMdast("heading", "# hi", (a) =>
  a?.type === "root" && a?.children?.[0]?.type === "heading");
await caseMdast("paragraph", "hello", (a) =>
  a?.children?.[0]?.type === "paragraph");

console.log("错误路径:");
await caseRawCode("bad json -> 400201", "markdown_to_html", "not json", 400201);
await caseRawCode("bad json -> 400202", "markdown_to_mdast", "{bad", 400202);
await caseRawCode("missing source -> 400201", "markdown_to_html", "{}", 400201);
await caseRawCode("missing source -> 400202", "markdown_to_mdast", "{}", 400202);

console.log("");
if (failed === 0) {
  console.log("全部通过");
} else {
  console.error(`${failed} 个用例失败`);
  Deno.exit(1);
}
