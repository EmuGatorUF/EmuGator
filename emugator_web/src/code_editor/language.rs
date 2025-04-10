use dioxus::signals::Readable;
use dioxus_logger::tracing::info;
use js_sys::{Array, Object};
use monaco::sys::{
    CancellationToken, IMarkdownString, IPosition,
    editor::ITextModel,
    languages::{self, Hover, HoverProvider, ILanguageExtensionPoint, LanguageConfiguration},
};
use wasm_bindgen::prelude::*;

use crate::interface::ASSEMBLED_PROGRAM;

use super::new_object;

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
    languages::register_hover_provider(language_id, &make_hover_provider());
}

fn make_hover_provider() -> HoverProvider {
    let provide_hover_fn = Closure::wrap(Box::new(
        move |model: ITextModel, position: IPosition, _token: CancellationToken| -> JsValue {
            let content = js_sys::Array::new();

            if let Some(word_info) = model.get_word_at_position(&position) {
                let word = word_info.word();

                // get static docs
                content.push(&new_md_string(format!("docs for {}", &word).as_str()));

                // get dynamic info based on the current program
                if let Some(program) = ASSEMBLED_PROGRAM.read().as_ref() {
                    info!("got program!");

                    // for labels, get the address
                    if let Some(symbol) = program.symbol_table.get(&word) {
                        content.push(&new_md_string(format!("{}", symbol).as_str()));
                    }
                }
            }

            let hover: Hover = new_object().into();
            hover.set_contents(&content);
            hover.into()
        },
    )
        as Box<dyn Fn(ITextModel, IPosition, CancellationToken) -> JsValue>);

    // Set the provideHover method on the object
    let provider: HoverProvider = new_object().into();
    let _ = js_sys::Reflect::set(
        &provider,
        &JsValue::from_str("provideHover"),
        provide_hover_fn.as_ref().unchecked_ref(),
    )
    .unwrap();

    // Keep the callback in memory forever
    provide_hover_fn.forget();

    provider
}

fn new_md_string(value: &str) -> IMarkdownString {
    let md_string: IMarkdownString = new_object().into();
    js_sys::Reflect::set(
        &md_string,
        &JsValue::from_str("value"),
        &JsValue::from_str(value),
    )
    .unwrap();
    md_string
}
