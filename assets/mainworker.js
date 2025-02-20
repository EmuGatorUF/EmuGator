import init from "/./dist/assets/dioxus/emu-gator.js";

init("/./dist/assets/dioxus/emu-gator_bg.wasm").then(wasm => {
    console.log("initializing WASM module in worker");
    wasm.init_wasm_module();
    console.log("TEST");
});
