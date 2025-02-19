import init from "/./dist/assets/dioxus/emu-gator.js";

init("/./dist/assets/dioxus/emu-gator_bg.wasm").then(wasm => {
    console.log("initializing WASM module in worker");
    wasm.init_wasm_module();
    console.log("TEST");
});

//init = require('/./dist/assets/dioxus/emu-gator.js');

//importScripts("/./dist/assets/dioxus/emu-gator.js");
//console.log("importing WASM module in worker");

// (async function () {
//     console.log("initializing WASM module in worker");
//     await wasm_bindgen({ module_or_path: "/dist/assets/dioxus/emu-gator_bg.wasm" });
//     await wasm_bindgen.init_wasm_module();
//     console.log("TEST");

//     self.onmessage = function ({ data }) {
//         wasm_bindgen[data]();
//     };
// })();