use js_sys::{Array, Object};
use monaco::sys::languages::{self, ILanguageExtensionPoint, LanguageConfiguration};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/tokenProvider.js")]
extern "C" {
    #[wasm_bindgen(js_name = "makeTokensProvider")]
    fn make_tokens_provider() -> Object;
}

pub fn register_riscv_language() {
    let language_id = "riscv";

    // create extension pointq
    let extension_point: ILanguageExtensionPoint = Object::new().unchecked_into();
    extension_point.set_id(language_id);

    // make configuration
    let cfg: LanguageConfiguration = Object::new().unchecked_into();
    let brackets = Array::new_with_length(1);
    {
        let pair = Array::new_with_length(2);
        pair.set(0, JsValue::from_str("("));
        pair.set(1, JsValue::from_str(")"));
        brackets.set(0, pair.into());
    }
    cfg.set_brackets(Some(&brackets));

    // get token provider from js file in assets
    let tokens_provider = make_tokens_provider();

    languages::register(&extension_point);
    languages::set_language_configuration(language_id, &cfg);
    languages::set_monarch_tokens_provider(language_id, &tokens_provider);
}
