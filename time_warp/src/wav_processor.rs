use {
  hound::{SampleFormat, WavReader, WavSpec, WavWriter},
  std::path::Path,
  thiserror::Error,
};

#[derive(Debug, Error)]
pub enum WavProcessingError {
  #[error("Hound error: {0}")]
  HoundError(#[from] hound::Error),

  #[error("Unsupported WAV format: {0}")]
  FormatError(String),
}

pub struct WavFileData {
  pub samples: Vec<(f32, f32)>,
  pub duration_in_samples: usize,
  pub duration_in_ms: f32,
}

#[derive(Clone)]
pub struct WavProcessor {
  sample_rate: f32,
}

impl WavProcessor {
  pub fn new(sample_rate: f32) -> Self {
    Self { sample_rate }
  }

  pub fn read_wav<'a, P: AsRef<Path>>(
    &self,
    file_path: P,
  ) -> Result<WavFileData, WavProcessingError> {
    let mut reader = WavReader::open(file_path)?;
    let spec = reader.spec();

    if spec.channels > 2 {
      return Err(WavProcessingError::FormatError(
        "Only mono or stereo WAV files are supported.".to_string(),
      ));
    }
    if spec.sample_rate != self.sample_rate as u32 {
      return Err(WavProcessingError::FormatError(
        "Samplerate doesn't match.".to_string(),
      ));
    }

    let samples: Vec<f32> = match spec.bits_per_sample {
      16 => reader
        .samples::<i16>()
        .map(|s| s.map(|v| v as f32 / -(i16::MIN as f32)))
        .collect::<Result<_, _>>()?,
      24 => reader
        .samples::<i32>()
        .map(|s| {
          s.map(|v| {
            let v = (v << 8) >> 8;
            v as f32 / 8_388_608.0 // divided by 2^23
          })
        })
        .collect::<Result<_, _>>()?,
      32 => reader.samples::<f32>().collect::<Result<_, _>>()?,
      _ => {
        return Err(WavProcessingError::FormatError(
          "Unsupported WAV bit depth.".to_string(),
        ))
      }
    };

    let stereo_samples: Vec<(f32, f32)> = if spec.channels == 1 {
      samples.iter().map(|sample| (*sample, *sample)).collect()
    } else {
      samples
        .chunks_exact(2)
        .map(|chunk| (chunk[0], chunk[1]))
        .collect()
    };
    let duration_in_samples = stereo_samples.len();
    let duration_in_ms = duration_in_samples as f32 / self.sample_rate * 1000.;

    Ok(WavFileData {
      samples: stereo_samples,
      duration_in_samples,
      duration_in_ms,
    })
  }

  pub fn write_wav<P: AsRef<Path>>(
    &self,
    samples: Vec<(f32, f32)>,
    file_path: P,
  ) -> Result<(), WavProcessingError> {
    let spec = WavSpec {
      channels: 2,
      sample_rate: self.sample_rate as u32,
      bits_per_sample: 32,
      sample_format: SampleFormat::Float,
    };

    let mut writer = WavWriter::create(file_path, spec)?;

    for &(left, right) in samples.iter() {
      writer.write_sample(left)?;
      writer.write_sample(right)?;
    }

    writer.finalize()?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::wav_processor::WavProcessor;
  use std::path::Path;

  #[test]
  fn should_read_wav_file() {
    let file_path = Path::new("src/wav_processor/read_example.wav");
    let wav_processor = WavProcessor::new(44100.);
    let result = wav_processor.read_wav(file_path);

    assert!(result.is_ok());
    match result {
      Ok(r) => {
        r.samples
          .iter()
          .zip([(-0.61001587, -0.61001587), (0.049987793, 0.049987793)])
          .for_each(|(actual, expected)| {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
          });
        assert_eq!(r.duration_in_ms, 0.36281177);
      }
      _ => (),
    }
  }

  #[test]
  fn should_write_wav_file() {
    let file_path = Path::new("src/wav_processor/write_example.wav");
    let samples_to_write = (0..16)
      .map(|x| {
        let sample = x as f32 / 16. * 2. - 1.;
        (sample, sample)
      })
      .collect::<Vec<(f32, f32)>>();
    let wav_processor = WavProcessor::new(44100.);
    let result = wav_processor.write_wav(samples_to_write.clone(), file_path);

    assert!(result.is_ok());
    let samples = wav_processor.read_wav(file_path);
    match samples {
      Ok(r) => {
        r.samples
          .iter()
          .zip(samples_to_write.iter())
          .for_each(|(actual, expected)| {
            assert_eq!(actual.0, expected.0);
            assert_eq!(actual.1, expected.1);
          });
        assert_eq!(r.duration_in_ms, 0.36281177);
      }
      Err(e) => {
        println!("{}", e);
      }
    }
  }
}
