pub fn ms_to_samples(ms: f32, sample_rate: f32) -> u32 {
    (ms as u32 * sample_rate as u32) / 1000
}
