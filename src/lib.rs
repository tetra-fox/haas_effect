mod sample_player;

use nih_plug::prelude::*;
use sample_player::SamplePlayer;
use std::sync::Arc;

const MAX_DELAY_MS: f32 = 100.0;
const O_O_BYTES: &[u8] = include_bytes!("O_O.bin");

#[derive(Enum, Debug, Clone, Copy, PartialEq)]
enum Channel {
    Left,
    Right,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq)]
enum Polarity {
    Normal,
    Inverted,
}

struct HaasEffect {
    params: Arc<HaasEffectParams>,
    delay_buffers: [Vec<f32>; 2],
    write_pos: usize,
    sample_rate: f32,
    current_delay_ms: f32,
    o_o: SamplePlayer,
}

#[derive(Params)]
struct HaasEffectParams {
    #[id = "channel"]
    pub channel: EnumParam<Channel>,

    #[id = "delay"]
    pub delay_ms: FloatParam,

    #[id = "smoothing"]
    pub smoothing_ms: FloatParam,

    #[id = "mix"]
    pub mix: FloatParam,

    #[id = "feedback"]
    pub feedback: FloatParam,

    #[id = "polarity"]
    pub polarity: EnumParam<Polarity>,

    #[id = "crossfeed"]
    pub crossfeed: FloatParam,

    #[id = "limiter"]
    pub limiter: BoolParam,

    #[id = "o_o"]
    pub o_o: BoolParam,
}

impl Default for HaasEffect {
    fn default() -> Self {
        Self {
            params: Arc::new(HaasEffectParams::default()),
            delay_buffers: [Vec::new(), Vec::new()],
            write_pos: 0,
            sample_rate: 44100.0,
            current_delay_ms: 20.0,
            o_o: SamplePlayer::from_f32_le_bytes(O_O_BYTES, 1),
        }
    }
}

impl Default for HaasEffectParams {
    fn default() -> Self {
        Self {
            channel: EnumParam::new("Channel", Channel::Left),

            delay_ms: FloatParam::new(
                "Delay",
                20.0,
                FloatRange::Skewed {
                    min: 0.0,
                    max: MAX_DELAY_MS,
                    factor: FloatRange::skew_factor(-1.33),
                },
            )
            .with_unit("ms")
            .with_step_size(0.01)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            smoothing_ms: FloatParam::new(
                "Smoothing",
                25.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 100.0,
                },
            )
            .with_unit("ms")
            .with_step_size(0.01)
            .with_value_to_string(formatters::v2s_f32_rounded(2)),

            mix: FloatParam::new("Mix", 1.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            feedback: FloatParam::new("Feedback", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            polarity: EnumParam::new("Polarity", Polarity::Normal),

            crossfeed: FloatParam::new("Crossfeed", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_unit("%")
                .with_value_to_string(formatters::v2s_f32_percentage(2))
                .with_string_to_value(formatters::s2v_f32_percentage()),

            limiter: BoolParam::new("Limiter", true),

            o_o: BoolParam::new("O_O", false),
        }
    }
}

impl Plugin for HaasEffect {
    const NAME: &'static str = env!("CARGO_PKG_NAME");
    const VENDOR: &'static str = "tetra";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "me@tetra.cool";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _ctx: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        let max_delay_samples = (self.sample_rate * MAX_DELAY_MS * 0.001).ceil() as usize + 1;

        self.delay_buffers = [vec![0.0; max_delay_samples], vec![0.0; max_delay_samples]];
        self.current_delay_ms = self.params.delay_ms.value();

        true
    }

    fn reset(&mut self) {
        for buf in &mut self.delay_buffers {
            buf.fill(0.0);
        }
        self.write_pos = 0;
        self.current_delay_ms = self.params.delay_ms.value();
        self.o_o.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let buffer_len = self.delay_buffers[0].len();
        if buffer_len == 0 {
            return ProcessStatus::Normal;
        }

        let mut delay_samples = 0usize;

        for mut channel_samples in buffer.iter_samples() {
            let mut iter = channel_samples.iter_mut();
            let (Some(l), Some(r)) = (iter.next(), iter.next()) else {
                continue;
            };
            self.o_o.tick(self.params.o_o.value(), &mut [l, r]);

            let target_delay_ms = self.params.delay_ms.value();
            let smoothing_ms = self.params.smoothing_ms.value();

            let alpha = if smoothing_ms <= 0.0 {
                1.0
            } else {
                1.0 - (-1.0 / (smoothing_ms * 0.001 * self.sample_rate)).exp()
            };
            self.current_delay_ms += alpha * (target_delay_ms - self.current_delay_ms);

            delay_samples = (self.current_delay_ms * 0.001 * self.sample_rate).round() as usize;
            delay_samples = delay_samples.min(buffer_len - 1);

            let read_pos = (self.write_pos + buffer_len - delay_samples) % buffer_len;
            let selected_ch = self.params.channel.value() as usize;
            let mix = self.params.mix.value();
            let feedback = self.params.feedback.value();
            let polarity = self.params.polarity.value();
            let crossfeed = self.params.crossfeed.value();
            let polarity_sign = match polarity {
                Polarity::Inverted => -1.0,
                Polarity::Normal => 1.0,
            };
            let no_delay = delay_samples == 0;
            let limiter = self.params.limiter.value();

            for (ch, sample) in channel_samples.iter_mut().enumerate() {
                let dry = *sample;
                let delayed = if no_delay {
                    dry
                } else {
                    self.delay_buffers[ch][read_pos]
                };

                let buf_val = if ch == selected_ch {
                    dry + delayed * feedback
                } else {
                    dry
                };
                self.delay_buffers[ch][self.write_pos] = if limiter {
                    buf_val.clamp(-1.0, 1.0)
                } else {
                    buf_val
                };

                if ch == selected_ch {
                    let wet = delayed * polarity_sign;
                    *sample = dry + mix * (wet - dry);
                } else {
                    let wet = if no_delay {
                        dry
                    } else {
                        self.delay_buffers[selected_ch][read_pos] * polarity_sign
                    };
                    *sample = dry + crossfeed * (wet - dry);
                }
            }

            self.write_pos = (self.write_pos + 1) % buffer_len;
        }

        ProcessStatus::Tail(delay_samples as u32)
    }
}

impl ClapPlugin for HaasEffect {
    const CLAP_ID: &'static str = "cool.tetra.haas_effect";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(env!("CARGO_PKG_DESCRIPTION"));
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Delay,
    ];
}

impl Vst3Plugin for HaasEffect {
    const VST3_CLASS_ID: [u8; 16] = *b"fa725404f01ac05d";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Spatial];
}

nih_export_clap!(HaasEffect);
nih_export_vst3!(HaasEffect);
