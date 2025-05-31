use std::array;

const SIZE: usize = 1024;

#[derive(Clone)]
pub struct RsqrtTable {
  values: [f32; SIZE],
}

impl RsqrtTable {
  pub fn new(min_gain: f32, max_gain: f32) -> Self {
    Self {
      values: array::from_fn(|i| {
        let x = i as f32 / SIZE as f32;
        let gain = x * (max_gain - min_gain) + min_gain;
        gain.recip().sqrt()
      }),
    }
  }

  pub fn get_value(&self, x: f32) -> f32 {
    let index = (((x - 1.) / 14. * SIZE as f32) as usize).clamp(0, SIZE);
    self.values[index]
  }
}
