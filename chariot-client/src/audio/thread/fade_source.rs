use rodio::source::TakeDuration;
use rodio::{Sample, Source};
use std::time::Duration;

pub struct FadeSource<I> {
    input: I,
    take_duration: Option<TakeDuration<I>>,
}

impl<I> FadeSource<I>
where
    I: Source + Clone,
    I::Item: Sample,
{
    pub fn new(input: I) -> Self {
        FadeSource {
            input,
            take_duration: None,
        }
    }

    /// Returns a reference to the inner source.
    #[inline]
    pub fn inner(&self) -> &I {
        &self.input
    }

    /// Returns a mutable reference to the inner source.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.input
    }

    pub fn is_fadeout(&self) -> bool {
        self.take_duration.is_some()
    }

    pub fn set_fadeout(&mut self, duration: Duration) {
        let mut take_duration = self.input.clone().take_duration(duration);
        take_duration.set_filter_fadeout();
        self.take_duration = Some(take_duration);
    }
}

impl<I> Iterator for FadeSource<I>
where
    I: Source + Clone,
    I::Item: Sample,
{
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        if let Some(ref mut take_duration) = &mut self.take_duration {
            take_duration.next()
        } else {
            self.input.next()
        }
    }
}

impl<I> Source for FadeSource<I>
where
    I: Iterator + Source + Clone,
    I::Item: Sample,
{
    fn current_frame_len(&self) -> Option<usize> {
        if let Some(ref take_duration) = &self.take_duration {
            take_duration.current_frame_len()
        } else {
            self.input.current_frame_len()
        }
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels()
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        if let Some(ref take_duration) = &self.take_duration {
            take_duration.total_duration()
        } else {
            self.input.total_duration()
        }
    }
}
