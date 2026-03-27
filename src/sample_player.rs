pub struct SamplePlayer {
    samples: Vec<f32>,
    channels: usize,
    frame: usize,
    active: bool,
    prev_trigger: bool,
}

impl SamplePlayer {
    pub fn new(samples: Vec<f32>, channels: usize) -> Self {
        Self {
            samples,
            channels,
            frame: 0,
            active: false,
            prev_trigger: false,
        }
    }

    pub fn from_f32_le_bytes(bytes: &[u8], channels: usize) -> Self {
        let samples = bytes
            .chunks_exact(std::mem::size_of::<f32>())
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();

        Self::new(samples, channels)
    }

    fn start(&mut self) {
        self.frame = 0;
        self.active = true;
    }

    fn stop(&mut self) {
        self.active = false;
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.active = false;
    }

    pub fn tick(&mut self, triggered: bool, channel_samples: &mut [&mut f32]) {
        if triggered && !self.prev_trigger {
            self.start();
        } else if !triggered && self.prev_trigger {
            self.stop();
        }
        self.prev_trigger = triggered;

        if self.active {
            let frame_offset = self.frame * self.channels;
            if frame_offset < self.samples.len() {
                for (ch, sample) in channel_samples.iter_mut().enumerate() {
                    let src_ch = ch.min(self.channels - 1);
                    **sample += self.samples[frame_offset + src_ch];
                }
                self.frame += 1;
            } else {
                self.active = false;
            }
        }
    }
}
