console.log("importing WASM module in worker");
importScripts("/dist/assets/dioxus/emu-gator.js");

(async function () {
    console.log("initializing WASM module in worker");
    await wasm_bindgen({ module_or_path: "/dist/assets/dioxus/emu-gator_bg.wasm" });
    await wasm_bindgen.init_wasm_module();
    console.log("TEST");

    self.onmessage = function ({ data }) {
        wasm_bindgen[data]();
    };
})();

// async function web_worker_srt() {
//     console.log("initializing WASM module in worker");
//     await wasm_bindgen({ module_or_path: "/dist/assets/dioxus/emu-gator_bg.wasm" });
//     await wasm_bindgen.init_wasm_module();
//     console.log("TEST");

//     self.onmessage = function ({ data }) {
//         wasm_bindgen[data]();
//     };
// };

// web_worker_srt();

// (async function () {
//     console.log("initializing WASM module in worker");
//     await wasm_bindgen({ module_or_path: "/./assets/dioxus/emu-gator_bg.wasm" });
//     await wasm_bindgen.create_thread_creator();
//     console.log("TEST");

//     self.onmessage = function ({ data }) {
//         wasm_bindgen[data]();
//     };
// })();

// import init from "./emu-gator.js";

// await init();

// export function initWASMWorker() {
//     console.log('Initializing worker');

// }

// initWASMWorker();