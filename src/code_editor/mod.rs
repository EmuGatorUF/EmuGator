use std::collections::BTreeSet;

use dioxus::prelude::*;

mod highlight;
mod monaco_editor;

use monaco::sys::{
    MarkerSeverity,
    editor::{self, IEditorMinimapOptions, IMarkerData},
};
use monaco_editor::MonacoEditor;

pub use highlight::register_riscv_language;
pub use monaco_editor::LineHighlight;
use wasm_bindgen::JsValue;

use crate::assembler::AssemblerError;

/// A wrapper around the Monaco editor with our expected functionality
#[component]
#[allow(non_snake_case)]
pub fn CodeEditor(
    mut source: Signal<String>,
    line_highlights: ReadOnlySignal<Vec<LineHighlight>>,
    breakpoints: Signal<BTreeSet<usize>>,
    assembler_errors: ReadOnlySignal<Vec<AssemblerError>>,
) -> Element {
    // basic model
    // TODO: support external changes to source being reflected in the model
    let mut model = use_signal(|| {
        monaco::api::TextModel::create(source.peek().as_str(), Some("riscv"), None).unwrap()
    });

    let mut source_sync = use_effect(move || {
        *source.write() = model().get_value();
    });

    let _model_listener = use_signal(move || {
        model.peek().on_did_change_content(move |_| {
            source_sync.mark_dirty();
        })
    });

    // basic options
    let options = use_signal(|| {
        let options = monaco::api::CodeEditorOptions::default()
            .with_automatic_layout(true)
            .with_builtin_theme(monaco::sys::editor::BuiltinTheme::VsDark)
            .to_sys_options();
        options.set_glyph_margin(Some(true));

        // disable the minimap
        let disable_minimap = IEditorMinimapOptions::default();
        disable_minimap.set_enabled(Some(false));
        options.set_minimap(Some(&disable_minimap));

        options
    });

    // set the error markers on the model
    use_effect(move || {
        let markers_arr = js_sys::Array::new();
        for err in assembler_errors.read().iter() {
            let marker: IMarkerData = new_object().into();
            marker.set_message(&err.error_message);
            marker.set_start_line_number(err.line_number as f64);
            marker.set_end_line_number(err.line_number as f64);
            marker.set_start_column(err.column as f64);
            marker.set_end_column((err.column + err.width) as f64);
            marker.set_severity(MarkerSeverity::Error);
            markers_arr.push(&marker);
        }

        editor::set_model_markers(model.write().as_ref(), "assembler", &markers_arr);
    });

    rsx! {
        MonacoEditor {
            model: model(),
            options: options(),
            line_highlights,
            breakpoints,
        }
    }
}

// Creates a new `JsValue`. Done for convenience and readability.
fn new_object() -> JsValue {
    js_sys::Object::new().into()
}
