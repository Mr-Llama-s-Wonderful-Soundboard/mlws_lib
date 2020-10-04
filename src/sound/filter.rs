use miniaudio::Sample;
use miniaudio::{Frames, FramesMut};

use std::sync::Arc;

use crate::utils::IdMap;

/// A filter is simply a function applied to the height of the sound wave
/// The sample filter is just applied to sigle values
/// The frame filter is applied to a whole frame (&mut [Sample]), an array of samples
#[derive(Clone)]
pub struct Filter<S>
where
    S: 'static + Sample + Copy,
{
    sample_filters: IdMap<Arc<Box<dyn Fn(S) -> S + Send + Sync>>>,
    frame_filters: IdMap<Arc<Box<dyn Fn(&mut [S]) + Send + Sync>>>,
}

impl<S> Filter<S>
where
    S: 'static + Sample + Copy,
{
    pub fn new() -> Self {
        Self {
            sample_filters: IdMap::new(),
            frame_filters: IdMap::new(),
        }
    }

    pub fn add_sample_filter<SampleFilter: Fn(S) -> S + Send + Sync + 'static>(
        &mut self,
        f: SampleFilter,
    ) -> usize {
        self.sample_filters.add(Arc::new(Box::new(f)))
    }

    pub fn add_frame_filter<FrameFilter: Fn(&mut [S]) + Send + Sync + 'static>(
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
        for frame in output.frames_mut() {
            for filter in self.frame_filters.iter() {
                filter(frame)
            }
        }
        for sample in output.as_samples_mut() {
            for filter in self.sample_filters.iter() {
                *sample = filter(*sample)
            }
        }
    }
}
