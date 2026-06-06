import { Editor } from "bytemd";
import gfm from "@bytemd/plugin-gfm";

import { header_tpl } from "./views/header.ts";
import { editor_tpl } from "./views/editor.ts";

const app = document.querySelector("#app") as HTMLElement;
// 插入头部模板
app.innerHTML = header_tpl + editor_tpl;

// 初始化编辑器
const editor = new Editor({
  target: document.querySelector("#scimark-editor") as HTMLElement,
  props: {
    value: "### Hello World!",
    placeholder: "请输入 Markdown...",
  },
  plugins: [gfm()]
});

document.querySelector(".bytemd-toolbar-left .octicon-mark-github")?.closest(
  "button",
).remove();

// 监听内容变化
editor.$on("change", (e: CustomEvent<{ value: string }>) => {
  console.log("最新内容：", e.detail.value);
});
