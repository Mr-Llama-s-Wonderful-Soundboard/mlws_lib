use miniaudio::Sample;

use std::marker::PhantomData;

use crate::utils::IdMap;

/// A filter is simply a function applied to the height of the sound wave
/// The sample filter is just applied to sigle values
/// The frame filter is applied to a whole frame (&mut [Sample]), an array of samples
pub struct Filter<S, SampleFilter, FrameFilter>
where
    S: Sample,
    SampleFilter: FnOnce(S) -> S,
    FrameFilter: Fn(&mut [S]),
{
    sample_filters: IdMap<SampleFilter>,
    frame_filters: IdMap<FrameFilter>,
    phantom: PhantomData<S>,
}

impl<S, SampleFilter, FrameFilter> Filter<S, SampleFilter, FrameFilter>
where
    S: Sample,
    SampleFilter: FnOnce(S) -> S,
    FrameFilter: Fn(&mut [S]),
{
    pub fn new() -> Self {
        Self {
            sample_filters: IdMap::new(),
            frame_filters: IdMap::new(),
            phantom: PhantomData,
        }
    }

    pub fn add_sample_filter(&mut self, f: SampleFilter) -> usize {
        self.sample_filters.add(f)
    }

    pub fn add_frame_filter(&mut self, f: FrameFilter) -> usize {
        self.frame_filters.add(f)
    }

    pub fn remove_sample_filter(&mut self, id: usize) -> Result<(), ()> {
        self.sample_filters.remove(id).ok_or(()).map(|_| ())
    }

    pub fn remove_frame_filter(&mut self, id: usize) -> Result<(), ()> {
        self.frame_filters.remove(id).ok_or(()).map(|_| ())
    }
}
