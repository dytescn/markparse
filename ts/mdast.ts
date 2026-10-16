import { callFfi, type ToMdastData } from "./ffi.ts";

// ============================================================
// markdown_to_mdast
// 入参: { source: string, options?: MarkdownOptions }
// 出参: { ast: unknown }
// ============================================================

const source = Deno.args[0] ?? "# hi\n\nhello world";

console.log("source:");
console.log(source);
console.log("---");

try {
  const data = await callFfi<ToMdastData>("markdown_to_mdast", { source });
  console.log("markdown_to_mdast -> ok");
  console.log(JSON.stringify(data.ast, null, 2));
} catch (err) {
  console.error("markdown_to_mdast 失败:", (err as Error).message);
  Deno.exit(1);
}

