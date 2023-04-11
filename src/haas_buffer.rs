use crate::util;
use rtrb::{RingBuffer, PushError, PopError, Producer, Consumer};
use std::usize;

pub struct HaasBuffer<T> {
    pub producer: Producer<T>,
    pub consumer: Consumer<T>,
    pub delay_ms: f32,
    pub sample_rate: f32,
}

impl<T> HaasBuffer<T>
where
    T: Default + Clone,
{
    pub fn new(delay_ms: f32, sample_rate: f32) -> Self {
        let size = util::ms_to_samples(delay_ms, sample_rate) as usize;
        let (mut producer, mut consumer) = RingBuffer::new(size);

        // for _ in 0..size {
        //     producer.push(T::default()).expect("zamn!!");
        // }

        Self {
            producer,
            consumer,
            delay_ms,
            sample_rate,
        }
    }

    pub fn resize(&mut self, ms_delta: f32) {
        let size = util::ms_to_samples(self.delay_ms + ms_delta, self.sample_rate) as usize;
        let (mut producer, mut consumer) = RingBuffer::new(size);

        producer.write_chunk(size);

        self.producer = producer;
        self.consumer = consumer;
    }

    pub fn push(&mut self, value: T) -> Result<(), PushError<T>> {
        self.producer.push(value)
    }

    pub fn pop(&mut self) -> Result<T, PopError> {
        self.consumer.pop()
    }

    pub fn reset(&mut self) {
    }
}
