use crate::{HyperclipParams, Mode};
use nih_plug::prelude::*;
use nih_plug_iced::widgets as nih_widgets;
use nih_plug_iced::*;
use std::sync::Arc;

pub fn default_state() -> Arc<IcedState> {
    IcedState::from_size(230, 500)
}

pub fn create(
    params: Arc<HyperclipParams>,
    editor_state: Arc<IcedState>,
) -> Option<Box<dyn Editor>> {
    create_iced_editor::<HyperclipEditor>(editor_state, (params,))
}

pub struct HyperclipEditor {
    params: Arc<HyperclipParams>,
    context: Arc<dyn GuiContext>,

    input_gain_state: nih_widgets::param_slider::State,
    output_gain_state: nih_widgets::param_slider::State,
    drive_state: nih_widgets::param_slider::State,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ParamUpdate(nih_widgets::ParamMessage),
    RadioSelected(Mode),
}

impl IcedEditor for HyperclipEditor {
    type Executor = executor::Default;
    type Message = Message;
    type InitializationFlags = (Arc<HyperclipParams>,);

    fn new(
        (params,): Self::InitializationFlags,
        context: Arc<dyn GuiContext>,
    ) -> (Self, nih_plug_iced::Command<Self::Message>) {
        let editor = Self {
            params,
            context,
            input_gain_state: Default::default(),
            output_gain_state: Default::default(),
            drive_state: Default::default(),
        };

        (editor, Command::none())
    }

    fn context(&self) -> &dyn GuiContext {
        self.context.as_ref()
    }

    fn update(
        &mut self,
        _window: &mut nih_plug_iced::WindowQueue,
        message: Self::Message,
    ) -> nih_plug_iced::Command<Self::Message> {
        match message {
            Message::ParamUpdate(message) => self.handle_param_message(message),
            Message::RadioSelected(choice) => {
                let setter = ParamSetter::new(self.context());
                setter.begin_set_parameter(&self.params.mode);
                setter.set_parameter(&self.params.mode, choice);
                setter.end_set_parameter(&self.params.mode);
            }
        }

        Command::none()
    }

    fn view(&mut self) -> nih_plug_iced::Element<'_, Self::Message> {
        let regular_text = |s: &str| {
            Text::new(s)
                .size(30)
                .height(40.into())
                .font(assets::NOTO_SANS_LIGHT)
                .horizontal_alignment(alignment::Horizontal::Center)
                .vertical_alignment(alignment::Vertical::Center)
        };

        let selected = Some(self.params.mode.value());
        let radio = |label: &str, mode: Mode| {
            nih_plug_iced::Radio::new(mode, label, selected, Message::RadioSelected)
        };

        let vertical_space = |amount: u16| {
            nih_plug_iced::Space::new(
                Length::Fill,
                Length::Units(amount)
            )
        };

        Column::new()
            .align_items(Alignment::Center)
            .push(
                Text::new("Hyperclip")
                    .size(40)
                    .height(50.into())
                    .font(assets::NOTO_SANS_REGULAR)
                    .width(Length::Fill)
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .vertical_alignment(alignment::Vertical::Bottom),
            )
            .push(vertical_space(13))

            .push(regular_text("Input gain"))
            .push(
                nih_widgets::ParamSlider::new(&mut self.input_gain_state, &self.params.input_gain)
                    .map(Message::ParamUpdate),
            )
            .push(vertical_space(13))

            .push(regular_text("Output gain"))
            .push(
                nih_widgets::ParamSlider::new(
                    &mut self.output_gain_state,
                    &self.params.output_gain,
                )
                .map(Message::ParamUpdate),
            )
            .push(vertical_space(13))

            .push(regular_text("Drive"))
            .push(
                nih_widgets::ParamSlider::new(&mut self.drive_state, &self.params.drive)
                    .map(Message::ParamUpdate),
            )
            .push(vertical_space(13))

            .push(regular_text("Mode"))
            .push(
                Column::new()
                    .align_items(Alignment::Fill)
                    .push(radio("Linear", Mode::Linear))
                    .push(radio("Exponential", Mode::Exponential))
                    .push(radio("Logarithmic", Mode::Logarithmic))
                    .push(radio("Sine", Mode::Sine)),
            )
            .into()
    }
}
