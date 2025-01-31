use std::collections::BTreeSet;

use dioxus::prelude::*;

mod highlight;
mod monaco_editor;

use monaco::sys::editor::IEditorMinimapOptions;
use monaco_editor::MonacoEditor;

pub use highlight::register_riscv_language;
pub use monaco_editor::LineHighlight;

/// A wrapper around the Monaco editor with our expected functionality
#[component]
#[allow(non_snake_case)]
pub fn CodeEditor(
    mut source: Signal<String>,
    line_highlights: ReadOnlySignal<Vec<LineHighlight>>,
    breakpoints: Signal<BTreeSet<usize>>,
) -> Element {
    // basic model
    // TODO: support external changes to source being reflected in the model
    let model = use_signal(|| {
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

    rsx! {
        MonacoEditor {
            model: model(),
            options: options(),
            line_highlights,
            breakpoints,
        }
    }
}
