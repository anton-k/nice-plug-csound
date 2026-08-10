use crate::params::{CsoundEditorState, CsoundParams};
use nice_plug::prelude::*;
use nice_plug_iced::iced::widget::{Slider, slider};

use nice_plug_iced::{EditorState, NiceGuiContext};

#[derive(Debug, Clone)]
pub enum Message {
    /// Sent when the application should poll parameters/meters and redraw.
    Poll,
    ParamChange {
        name: String,
        value: f32,
    },
}

pub struct CsoundGui {
    /// The editor state is stored inside of a wrapper which allows the
    /// state to persist across editor opens.
    pub editor_state: EditorState<CsoundEditorState>,

    /// A handle that can be used to request operations from nice-plug, like
    /// resizing the window.
    #[allow(unused)]
    pub nice_ctx: NiceGuiContext,
}

impl CsoundGui {
    pub fn new(editor_state: EditorState<CsoundEditorState>, nice_ctx: NiceGuiContext) -> Self {
        Self {
            editor_state,
            nice_ctx,
        }
    }

    pub fn update(&mut self, message: Message) {
        let setter = self.nice_ctx.param_setter();
        let params = &self.editor_state.params;

        match message {
            Message::Poll => {}
            Message::ParamChange { name, value } => {
                if let Some(size) = params.get(&name) {
                    setter.begin_set_parameter(size);
                    setter.set_parameter_normalized(size, value);
                    setter.end_set_parameter(size);
                }
            }
        }
    }
}

pub fn uni_slider<'a, Theme, Params: CsoundParams>(
    name: &'static str,
    params: &Params,
) -> Slider<'a, f32, Message, Theme>
where
    Theme: slider::Catalog + 'a,
{
    slider(
        0.0..=1.0,
        if let Some(param) = params.get(name) {
            param.modulated_normalized_value()
        } else {
            1.0
        },
        |value| Message::ParamChange {
            name: name.to_string(),
            value,
        },
    )
}
