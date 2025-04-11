use super::data_views::DataView;
use super::instruction_views::InstructionView;
use dioxus::prelude::*;
use emugator_core::assembler::AssembledProgram;
use emugator_core::emulator::AnyEmulatorState;

#[derive(PartialEq, Clone, Copy)]
pub enum MemoryViewType {
    Instruction,
    Data,
}

#[component]
#[allow(non_snake_case)]
pub fn MemoryView(
    assembled_program: ReadOnlySignal<Option<AssembledProgram>>,
    emulator_state: ReadOnlySignal<Option<AnyEmulatorState>>,
) -> Element {
    let mut view_type = use_signal(|| MemoryViewType::Instruction);

    rsx! {
        div { class: "h-full flex flex-col overflow-hidden",
            // View selector buttons
            div { class: "flex gap-4 mb-2 flex-shrink-0",
                button {
                    class: "text-lg font-mono font-bold text-gray-200 hover:text-gray-300 transition-colors cursor-pointer",
                    style: if *view_type.read() == MemoryViewType::Instruction { "text-decoration: underline" } else { "" },
                    onclick: move |_| view_type.set(MemoryViewType::Instruction),
                    "Instruction Memory"
                }
                span { class: "text-lg font-mono font-bold text-gray-200", "/" }
                button {
                    class: "text-lg font-mono font-bold text-gray-200 hover:text-gray-300 transition-colors cursor-pointer",
                    style: if *view_type.read() == MemoryViewType::Data { "text-decoration: underline" } else { "" },
                    onclick: move |_| view_type.set(MemoryViewType::Data),
                    "Data Memory"
                }
            }

            div { class: "flex-grow overflow-hidden",
                match *view_type.read() {
                    MemoryViewType::Instruction => rsx! {
                        InstructionView { assembled_program, emulator_state }
                    },
                    MemoryViewType::Data => rsx! {
                        DataView { assembled_program, emulator_state }
                    },
                }
            }
        }
    }
}
