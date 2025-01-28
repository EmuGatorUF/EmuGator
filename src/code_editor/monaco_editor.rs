use std::{collections::BTreeSet, ops::Deref, vec};

use dioxus::prelude::*;
use monaco::{
    api::{CodeEditor as MonacoController, DisposableClosure, TextModel},
    sys::{
        editor::{
            IEditorMouseEvent, IModelDecorationOptions, IModelDeltaDecoration,
            IStandaloneEditorConstructionOptions, MouseTargetType,
        },
        IRange, Range,
    },
};
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, PartialEq, Debug)]
pub struct LineHighlight {
    pub line: usize,
    pub css_class: &'static str,
}

/// The monaco editor directly wrapped
#[component]
#[allow(non_snake_case)]
pub fn MonacoEditor(
    options: ReadOnlySignal<Option<IStandaloneEditorConstructionOptions>>,
    model: ReadOnlySignal<Option<TextModel>>,
    line_highlights: ReadOnlySignal<Vec<LineHighlight>>,
    breakpoints: Signal<BTreeSet<usize>>,
) -> Element {
    let mut editor = use_signal::<Option<MonacoController>>(|| None);
    let element_id = "monaco-editor";

    let mut curr_decorations = use_signal(|| js_sys::Array::new());

    let mut mouse_handlers: Signal<Vec<DisposableClosure<dyn FnMut(IEditorMouseEvent)>>> =
        use_signal(|| vec![]);

    let breakpoint_hover_line: Signal<Option<usize>> = use_signal(|| None);

    // create editor
    use_effect(move || {
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(element_id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let options = options.read().deref().clone();
            let controller = MonacoController::create(&el, options);
            mouse_handlers
                .write()
                .push(controller.on_mouse_move(move |e| on_mouse_move(e, breakpoint_hover_line)));
            mouse_handlers
                .write()
                .push(controller.on_mouse_down(move |e| on_mouse_click(e, breakpoints)));
            *editor.write() = Some(controller);
        }
    });

    // handle model changes
    use_effect(move || {
        if let Some(editor_instance) = editor.write().as_mut() {
            // Update model if changed
            if let Some(model) = &*model.read() {
                editor_instance.set_model(model);
            }
        }
    });

    // handle decorators
    use_effect(move || {
        if let Some(editor_instance) = editor.write().as_mut() {
            if let Some(model) = editor_instance.get_model().as_ref() {
                let new_decor = js_sys::Array::new();

                // find new highlights
                for line_highlight in line_highlights.read().iter() {
                    new_decor.push(&line_decoration(
                        line_highlight.line,
                        line_highlight.css_class,
                    ));
                }

                // add breakpoint hover dot
                if let Some(line) = *breakpoint_hover_line.read() {
                    if !breakpoints.read().contains(&line) {
                        new_decor.push(&breakpoint_decoration(line, "monaco-breakpoint-preview"));
                    }
                }

                // add breakpoints
                for line in breakpoints.read().iter() {
                    new_decor.push(&breakpoint_decoration(*line, "monaco-breakpoint"));
                }

                // apply highlights
                let applied =
                    model
                        .as_ref()
                        .delta_decorations(&curr_decorations.peek(), &new_decor, None);

                // store highlights for next delta
                *curr_decorations.write() = applied;
            }
        }
    });

    rsx! {
        div { id: element_id, style: "width: 100%; height: 100%;" }
    }
}

fn on_mouse_move(e: IEditorMouseEvent, mut breakpoint_hover_line: Signal<Option<usize>>) {
    let hover_target = e.target().type_();
    let on_margin_or_number = hover_target == MouseTargetType::GutterGlyphMargin
        || hover_target == MouseTargetType::GutterLineNumbers;
    if on_margin_or_number {
        if let Some(line) = e.target().position().map(|p| p.line_number() as usize) {
            breakpoint_hover_line.set(Some(line));
        }
    } else {
        breakpoint_hover_line.set(None);
    }
}

fn on_mouse_click(e: IEditorMouseEvent, mut breakpoints: Signal<BTreeSet<usize>>) {
    let on_margin = e.target().type_() == MouseTargetType::GutterGlyphMargin;
    if on_margin {
        if let Some(line) = e.target().position().map(|p| p.line_number() as usize) {
            breakpoints.write().insert(line);
        }
    }
}

fn line_decoration(line_number: usize, class: &'static str) -> IModelDeltaDecoration {
    let decoration: IModelDeltaDecoration = new_object().into();
    let range = Range::new(line_number as f64, 0.0, line_number as f64, 1.0);
    decoration.set_range(&IRange::from(range.dyn_into::<JsValue>().unwrap()));

    let options: IModelDecorationOptions = new_object().into();
    options.set_is_whole_line(Some(true));
    options.set_z_index(Some(9999.0));
    options.set_class_name(Some(class));

    decoration.set_options(&options);

    decoration.into()
}

fn breakpoint_decoration(line_number: usize, class: &'static str) -> IModelDeltaDecoration {
    let decoration: IModelDeltaDecoration = new_object().into();
    let range = Range::new(line_number as f64, 1.0, line_number as f64, 1.0);
    decoration.set_range(&IRange::from(range.dyn_into::<JsValue>().unwrap()));

    let options: IModelDecorationOptions = new_object().into();
    options.set_glyph_margin_class_name(Some(class));

    decoration.set_options(&options);

    decoration.into()
}

// Creates a new `JsValue`. Done for convenience and readability.
fn new_object() -> JsValue {
    js_sys::Object::new().into()
}
