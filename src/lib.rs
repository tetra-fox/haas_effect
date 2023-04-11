mod channel;
mod custom_formatters;
mod editor;
mod haas_buffer;
mod util;

use channel::Channel;
use haas_buffer::HaasBuffer;
use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;

use std::sync::Arc;

pub struct HaasHimselfPlugin {
    params: Arc<HaasHimselfPluginParams>,
    sample_rate: f32,
    buffer: HaasBuffer<(f32, f32)>,
}

#[derive(Params)]
struct HaasHimselfPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,
    #[id = "delay"]
    pub delay: FloatParam,
    #[id = "channel"]
    pub channel: EnumParam<Channel>,
}

impl Default for HaasHimselfPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(HaasHimselfPluginParams::default()),
            sample_rate: 44100.,
            buffer: HaasBuffer::new(HaasHimselfPluginParams::default().delay.default_plain_value(), 44100.),
        }
    }
}

impl Default for HaasHimselfPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            delay: FloatParam::new("Delay", 100., FloatRange::Linear { min: 0., max: 200. })
                .with_value_to_string(custom_formatters::v2s_f32_ms_then_s(2))
                .with_string_to_value(custom_formatters::s2v_f32_ms_then_s()),

            channel: EnumParam::new("Channel", Channel::Right),
        }
    }
}

impl Plugin for HaasHimselfPlugin {
    const NAME: &'static str = "HaasHimself";
    const VENDOR: &'static str = "tetra";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "me@tetra.cool";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.params.editor_state.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;

        let buffer_size = util::ms_to_samples(self.params.delay.value(), buffer_config.sample_rate);

        // self.buffer
        //     .resize(self.params.delay.value(), buffer_config.sample_rate);

        for _ in 0..buffer_size {
            self.buffer.push((0., 0.)).unwrap();
        }

        true
    }

    fn reset(&mut self) {
        self.buffer.reset();
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        if self.params.delay.value() <= 0. {
            // bypass
            return ProcessStatus::Normal;
        }

        // self.buffer.resize(self.params.delay.value(), self.sample_rate);
        for mut channel_samples in buffer.iter_samples() {
            self.buffer
                .push((
                    *channel_samples.get_mut(0).unwrap(),
                    *channel_samples.get_mut(1).unwrap(),
                ))
                .unwrap();

            if self.params.channel.value() == Channel::Left {
                *channel_samples.get_mut(0).unwrap() = self.buffer.pop().unwrap().0;
            } else if self.params.channel.value() == Channel::Right {
                *channel_samples.get_mut(1).unwrap() = self.buffer.pop().unwrap().1;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for HaasHimselfPlugin {
    const CLAP_ID: &'static str = "cool.tetra.haashimself";
    const CLAP_DESCRIPTION: Option<&'static str> = Some(env!("CARGO_PKG_DESCRIPTION"));
    const CLAP_MANUAL_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for HaasHimselfPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"Its_Haas_Himself"; // must be 16 chars

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Spatial,
        Vst3SubCategory::Delay,
    ];
}

nih_export_clap!(HaasHimselfPlugin);
nih_export_vst3!(HaasHimselfPlugin);
