import init from "./pkg/nexus_compiler.js";

const wasm = await init('./pkg/nexus_compiler_bg.wasm');
wasm.nexus_init();