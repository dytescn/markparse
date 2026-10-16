import { callFfi, type ToHtmlData } from "./ffi.ts";

// ============================================================
// markdown_to_html
// 入参: { source: string, options?: MarkdownOptions }
// 出参: { html: string }
// ============================================================

const source = Deno.args[0] ?? "# hi\n\n**bold** and *italic*";

console.log("source:");
console.log(source);
console.log("---");

try {
  const data = await callFfi<ToHtmlData>("markdown_to_html", { source });
  console.log("markdown_to_html -> ok");
  console.log(data.html);
} catch (err) {
  console.error("markdown_to_html 失败:", (err as Error).message);
  Deno.exit(1);
}

