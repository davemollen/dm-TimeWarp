use {
  rubato::{FftFixedIn, Resampler},
  std::{fs::File, path::Path},
  symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
  },
  thiserror::Error,
};

#[derive(Debug, Error)]
pub enum AudioFileProcessingError {
  #[error("Symphonia error: {0}")]
  SymphoniaError(#[from] symphonia::core::errors::Error),

  #[error("Read error: {0}")]
  ReadError(String),

  #[error("Resample error: {0}")]
  ResamplerConstructionError(#[from] rubato::ResamplerConstructionError),

  #[error("Resample error: {0}")]
  ResampleError(#[from] rubato::ResampleError),
}

pub struct AudioFileData {
  pub samples: Vec<f32>,
  pub duration_in_samples: usize,
  pub duration_in_ms: f32,
}

#[derive(Clone)]
pub struct AudioFileProcessor {
  sample_rate: f32,
}

impl AudioFileProcessor {
  pub fn new(sample_rate: f32) -> Self {
    Self { sample_rate }
  }

  pub fn read<'a, P: AsRef<Path>>(
    &self,
    file_path: P,
  ) -> Result<AudioFileData, AudioFileProcessingError> {
    // Create a media source. Note that the MediaSource trait is automatically implemented for File,
    // among other types.
    let file = Box::new(File::open(file_path).unwrap());

    // Create the media source stream using the boxed media source from above.
    let mss = MediaSourceStream::new(file, Default::default());

    // Create a hint to help the format registry guess what format reader is appropriate. In this
    // example we'll leave it empty.
    let hint = Hint::new();

    // Use the default options when reading and decoding.
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    let decoder_opts: DecoderOptions = Default::default();

    // Probe the media source stream for a format.
    let probed =
      symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

    // Get the format reader yielded by the probe operation.
    let mut format = probed.format;

    // Get the default track.
    let track = format.default_track().unwrap();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
      .make(&track.codec_params, &decoder_opts)
      .unwrap();

    // Store the track identifier, we'll use it to filter packets.
    let track_id = track.id;

    let chunk_size = 1024;
    let mut sample_buf = None;
    let mut samples: Vec<f32> = Vec::new();
    let mut resampler: Option<FftFixedIn<f32>> = None;
    let mut resample_buffer: Vec<f32> = Vec::new();

    loop {
      // Get the next packet from the format reader.
      let packet = match format.next_packet() {
        Ok(p) => p,
        Err(symphonia::core::errors::Error::IoError(e)) => {
          if e.kind() == std::io::ErrorKind::UnexpectedEof {
            break; // EOF reached
          }
          return Err(AudioFileProcessingError::SymphoniaError(
            symphonia::core::errors::Error::IoError(e),
          ));
        }
        Err(e) => return Err(AudioFileProcessingError::SymphoniaError(e)),
      };

      // If the packet does not belong to the selected track, skip it.
      if packet.track_id() != track_id {
        continue;
      }

      // Decode the packet into audio samples, ignoring any decode errors.
      let audio_buf = decoder.decode(&packet)?;

      let frames = audio_buf.frames();
      let spec = audio_buf.spec();
      let channels = spec.channels.count();
      let sample_rate = spec.rate as f32;

      // The decoded audio samples may now be accessed via the audio buffer if per-channel
      // slices of samples in their native decoded format is desired. Use-cases where
      // the samples need to be accessed in an interleaved order or converted into
      // another sample format, or a byte buffer is required, are covered by copying the
      // audio buffer into a sample buffer or raw sample buffer, respectively. In the
      // example below, we will copy the audio buffer into a sample buffer in an
      // interleaved order while also converting to a f32 sample format.

      // If this is the *first* decoded packet, create a sample buffer matching the
      // decoded audio buffer format.
      if sample_buf.is_none() {
        // Get the capacity of the decoded buffer. Note: This is capacity, not length!
        let duration = audio_buf.capacity() as u64;

        // Create the f32 sample buffer.
        sample_buf = Some(SampleBuffer::<f32>::new(duration, *spec));
      }

      // Copy the decoded audio buffer into the sample buffer in an interleaved format.
      if let Some(buf) = &mut sample_buf {
        buf.copy_interleaved_ref(audio_buf);
        let interleaved_samples = &buf.samples()[..frames * channels];
        let deinterleaved_samples: Vec<f32> = match channels {
          1 => Ok(interleaved_samples.to_vec()),
          2 => Ok(
            interleaved_samples
              .chunks_exact(2)
              .map(|chunk| (chunk[0] + chunk[1]) * 0.5)
              .collect(),
          ),
          _ => Err(AudioFileProcessingError::ReadError(
            "Only mono and stereo audio files are supported".to_string(),
          )),
        }?;

        if self.sample_rate != sample_rate {
          // Initialize resampler
          if resampler.is_none() {
            resampler = Some(FftFixedIn::<f32>::new(
              sample_rate as usize,
              self.sample_rate as usize,
              chunk_size,
              2, // number of FFT blocks per processing call
              1,
            )?);
          }

          // Buffer input samples for chunked processing
          resample_buffer.extend_from_slice(&deinterleaved_samples);

          // Process full chunks from resample_buffer
          // while is needed here to avoid unnecessary latency when multiple chunks are ready at once.
          while resample_buffer.len() >= chunk_size {
            let chunk: Vec<f32> = resample_buffer.drain(..chunk_size).collect();
            let resampled = resampler.as_mut().unwrap().process(&vec![chunk], None)?;
            samples.extend_from_slice(&resampled[0]);
          }
        } else {
          // No resampling needed
          samples.extend_from_slice(&deinterleaved_samples);
        }
      };
    }

    // After finishing reading all packets, flush leftover samples in resample_buffer if resampling
    if let Some(r) = &mut resampler {
      if !resample_buffer.is_empty() {
        // Pad the last chunk with zeros
        resample_buffer.resize(chunk_size, 0.0);
        let resampled = r.process(&vec![&resample_buffer], None)?;
        samples.extend_from_slice(&resampled[0]);
        resample_buffer.clear();
      }
    }
    let duration_in_samples = samples.len();
    let duration_in_ms = duration_in_samples as f32 / self.sample_rate * 1000.;

    return Ok(AudioFileData {
      samples,
      duration_in_samples,
      duration_in_ms,
    });
  }
}

#[cfg(test)]
mod tests {
  use crate::audio_file_processor::AudioFileProcessor;
  use std::path::Path;

  #[test]
  fn should_read_audio_file() {
    let file_path = Path::new("src/audio_file_processor/read_example.wav");
    let audio_file_processor = AudioFileProcessor::new(44100.);
    let result = audio_file_processor.read(file_path);

    assert!(result.is_ok());
    match result {
      Ok(r) => {
        r.samples
          .iter()
          .zip([-0.61001587, 0.049987793])
          .for_each(|(actual, expected)| {
            assert_eq!(*actual, expected);
          });
        assert_eq!(r.duration_in_ms, 0.36281177);
      }
      _ => (),
    }
  }
}
