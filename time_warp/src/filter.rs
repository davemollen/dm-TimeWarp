mod one_pole_filter;
use one_pole_filter::{FilterType, OnePoleFilter};

pub struct Filter {
  lowpass_filter: OnePoleFilter,
  highpass_filter: OnePoleFilter,
}

impl Filter {
  pub fn new(sample_rate: f32) -> Self {
    Self {
      lowpass_filter: OnePoleFilter::new(sample_rate),
      highpass_filter: OnePoleFilter::new(sample_rate),
    }
  }

  pub fn set_cutoff_frequencies(&mut self, highpass_freq: f32, lowpass_freq: f32) {
    self.highpass_filter.set_cutoff_freq(highpass_freq);
    self.lowpass_filter.set_cutoff_freq(lowpass_freq);
  }

  pub fn process(&mut self, x: f32) -> f32 {
    let highpass_out = self.highpass_filter.process(x, FilterType::Highpass);
    self
      .lowpass_filter
      .process(highpass_out, FilterType::Lowpass)
  }
}
