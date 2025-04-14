use std::{borrow::Cow, collections::HashMap};

use dioxus::signals::Readable;
use dioxus_logger::tracing::info;
use js_sys::{Array, Object};
use monaco::sys::{
    CancellationToken, IMarkdownString, IPosition,
    editor::ITextModel,
    languages::{self, Hover, HoverProvider, ILanguageExtensionPoint, LanguageConfiguration},
};
use serde::Deserialize;
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

#[derive(Deserialize)]
pub struct DocEntry<'a> {
    #[serde(borrow)]
    pub format: Cow<'a, str>,
    #[serde(borrow)]
    pub desc: Cow<'a, str>,
    #[serde(borrow)]
    pub example: Cow<'a, str>,
}

pub const DOCS: &str = include_str!("../../assets/docs.json");

fn get_word_at_position(line: &str, col: usize) -> &str {
    let word_end =
        |c: char| c.is_whitespace() || c.is_ascii_punctuation() && !['.', '_'].contains(&c);
    let start = line[..col].rfind(word_end).map(|i| i + 1).unwrap_or(0);
    let end = line[col..]
        .find(word_end)
        .map(|i| i + col)
        .unwrap_or(line.len());
    &line[start..end]
}

fn make_hover_provider() -> HoverProvider {
    let docs: HashMap<&'static str, DocEntry<'static>> =
        serde_json::from_str(DOCS).expect("failed to parse docs.json");

    let provide_hover_fn = Closure::wrap(Box::new(
        move |model: ITextModel, position: IPosition, _token: CancellationToken| -> JsValue {
            let content = js_sys::Array::new();

            let line = model.get_line_content(position.line_number());
            let col = position.column() as usize;

            let comment_start = line.find('#').unwrap_or(line.len());

            // ignore comments
            if col < comment_start {
                // get the word at the position
                let word = get_word_at_position(&line, col);

                info!("getting hover docs for '{}'", word);

                if !word.is_empty() {
                    // get static docs
                    if let Some(doc) = docs.get(word) {
                        content.push(&new_md_string(&format!(
                            "**{}**\n\n{}\n\n_Example:_\n```riscv\n{}\n```\n",
                            doc.format, doc.desc, doc.example
                        )));
                    }

                    // get dynamic info based on the current program
                    if let Some(program) = ASSEMBLED_PROGRAM.read().as_ref() {
                        // for labels, get the address
                        if let Some(symbol) = program.symbol_table.get(word) {
                            content.push(&new_md_string(format!("{}", symbol).as_str()));
                        }
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
