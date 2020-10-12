use miniaudio::{Sample, Format};
use miniaudio::{Frames, FramesMut};
use super::sample::Sample as SelfSample;

use std::sync::Arc;

use crate::utils::IdMap;

/// A filter is simply a function applied to the height of the sound wave
/// The sample filter is just applied to sigle values
/// The frame filter is applied to a whole frame (&mut [Sample]), an array of samples
#[derive(Clone)]
pub struct Filter
{
    sample_filters: IdMap<Arc<Box<dyn Fn(f32) -> f32 + Send + Sync>>>,
    frame_filters: IdMap<Arc<Box<dyn Fn(Vec<&mut f32>) + Send + Sync>>>,
}

impl Filter
{
    pub fn new() -> Self {
        Self {
            sample_filters: IdMap::new(),
            frame_filters: IdMap::new(),
        }
    }

    pub fn add_sample_filter<SampleFilter: Fn(f32) -> f32 + Send + Sync + 'static>(
        &mut self,
        f: SampleFilter,
    ) -> usize {
        self.sample_filters.add(Arc::new(Box::new(f)))
    }

    pub fn add_frame_filter<FrameFilter: Fn(Vec<&mut f32>) + Send + Sync + 'static>(
        &mut self,
        f: FrameFilter,
    ) -> usize {
        self.frame_filters.add(Arc::new(Box::new(f)))
    }

    pub fn remove_sample_filter(&mut self, id: usize) -> Result<(), ()> {
        self.sample_filters.remove(id).ok_or(()).map(|_| ())
    }

    pub fn remove_frame_filter(&mut self, id: usize) -> Result<(), ()> {
        self.frame_filters.remove(id).ok_or(()).map(|_| ())
    }

    pub fn apply(&self, input: &Frames, output: &mut FramesMut) {
        output.as_bytes_mut().copy_from_slice(input.as_bytes());
        match output.format() {
            Format::F32 => {
                for frame in output.frames_mut::<f32>() {
                    for filter in self.frame_filters.iter() {
                        filter(frame.iter_mut().collect())
                    }
                }
                for sample in output.as_samples_mut::<f32>() {
                    for filter in self.sample_filters.iter() {
                        *sample = filter(*sample)
                    }
                }
            }

            Format::S16 => {
                for frame in output.frames_mut::<i16>() {
                    let mut frame_f32: Vec<f32> = frame.iter().map(|x|x.to_f32()).collect();
                    for filter in self.frame_filters.iter() {
                        filter(frame_f32.iter_mut().collect())
                    }
                    for (x, y) in frame.iter_mut().zip(frame_f32) {
                        *x = y.to_i16()
                    }
                }
                for sample in output.as_samples_mut::<i16>() {
                    for filter in self.sample_filters.iter() {
                        *sample = filter(sample.to_f32()).to_i16()
                    }
                }
            }

            f => {
                panic!("Unexpected format: {:?}", f)
            }
        }
        
    }
}
