use nih_plug::prelude::*;
use nih_plug_iced::IcedState;
use std::sync::Arc;

mod editor;

#[derive(Enum, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Mode {
    Linear,
    Exponential,
    Logarithmic,
    Sine,
}

#[derive(Params)]
pub struct HyperclipParams {
    #[id = "input-gain"]
    pub input_gain: FloatParam,

    #[id = "output-gain"]
    pub output_gain: FloatParam,

    #[id = "drive"]
    pub drive: FloatParam,

    #[id = "mode"]
    pub mode: EnumParam<Mode>,

    #[persist = "editor-state"]
    editor_state: Arc<IcedState>,
}

impl Default for HyperclipParams {
    fn default() -> Self {
        let gain = |s: &str| {
            let min = -30.0;
            let max = 30.0;

            FloatParam::new(
                s,
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(min),
                    max: util::db_to_gain(max),
                    factor: FloatRange::gain_skew_factor(min, max),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db())
        };

        let drive = FloatParam::new("Drive", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
            .with_smoother(SmoothingStyle::Linear(10.0))
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage());

        let mode = EnumParam::<Mode>::new("Mode", Mode::Linear);
        Self {
            input_gain: gain("Input gain"),
            output_gain: gain("Output gain"),
            drive,
            mode,
            editor_state: editor::default_state(),
        }
    }
}

struct Hyperclip {
    params: Arc<HyperclipParams>,
}

impl Default for Hyperclip {
    fn default() -> Self {
        let params = Arc::new(HyperclipParams::default());

        Self { params }
    }
}

impl Plugin for Hyperclip {
    const NAME: &'static str = "Hyperclip";
    const VENDOR: &'static str = "n0emo";
    const URL: &'static str = "https://github.com/n0emo/hyperclip";
    const EMAIL: &'static str = "dev_n0emo@tuta.io";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    type SysExMessage = ();

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for sample in buffer.iter_samples() {
            let input_gain = self.params.input_gain.smoothed.next();
            let output_gain = self.params.output_gain.smoothed.next();
            let drive = self.params.drive.smoothed.next();
            let mode = self.params.mode.value();

            for channel in sample {
                let mut x = *channel;
                let sign = x.signum();
                x *= input_gain * sign;

                // y = (-1)^floor(arg)(mod(arg, 2) - 1) + 1
                let ax = x * (drive * 4.0 + 1.0);
                let arg = match mode {
                    Mode::Linear => ax,
                    Mode::Exponential => ax.exp() - 1.0,
                    Mode::Logarithmic => (ax + 1.0).ln(),
                    Mode::Sine => ax.sin(),
                };
                let pow_mul = (-1 as i32).pow(arg.floor() as u32) as f32;
                x = pow_mul * (arg % 2.0 - 1.0) + 1.0;

                x *= output_gain * sign;
                *channel = x;
            }
        }
        ProcessStatus::Normal
    }
}

impl Vst3Plugin for Hyperclip {
    const VST3_CLASS_ID: [u8; 16] = *b"n0emo-Hyperclip-";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Distortion];
}

impl ClapPlugin for Hyperclip {
    const CLAP_ID: &'static str = "org.n0emo.hyperclip";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(env!("CARGO_PKG_DESCRIPTION"));
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Distortion,
    ];
}

nih_export_vst3!(Hyperclip);
nih_export_clap!(Hyperclip);
