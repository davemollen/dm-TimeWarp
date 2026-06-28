use {
  rubato::{audioadapter_buffers::direct::InterleavedSlice, Fft, FixedSync, Resampler},
  std::{fs::File, path::Path},
  symphonia::core::{
    codecs::audio::AudioDecoderOptions,
    formats::{probe::Hint, FormatOptions, TrackType},
    io::MediaSourceStream,
    meta::MetadataOptions,
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

  #[error("Resample buffer size error: {0}")]
  SizeError(#[from] rubato::audioadapter_buffers::SizeError),

  #[error("Sample rate not found.")]
  SampleRateError,
}

pub struct AudioFileData {
  pub samples: Vec<f32>,
  pub duration_in_samples: usize,
  pub duration_in_ms: f32,
}

#[derive(Clone)]
pub struct AudioFileProcessor {
  host_sample_rate: usize,
  max_size: usize,
}

impl AudioFileProcessor {
  pub fn new(sample_rate: f32, max_size: usize) -> Self {
    Self {
      host_sample_rate: sample_rate as usize,
      max_size,
    }
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
    let fmt_opts: FormatOptions = Default::default();
    let meta_opts: MetadataOptions = Default::default();
    let dec_opts: AudioDecoderOptions = Default::default();

    // Probe the media source stream for a format.
    let mut format = symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;

    // Get the default audio track.
    let track = format.default_track(TrackType::Audio).unwrap();

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs()
      .make_audio_decoder(
        track.codec_params.as_ref().unwrap().audio().unwrap(),
        &dec_opts,
      )
      .unwrap();

    // Store the track identifier, we'll use it to filter packets.
    let track_id = track.id;

    // Read the track samplerate
    let file_sample_rate = track
      .codec_params
      .as_ref()
      .and_then(|params| {
        params
          .audio()
          .and_then(|audio_params| audio_params.sample_rate.map(|sr| sr as usize))
      })
      .ok_or_else(|| AudioFileProcessingError::SampleRateError)?;

    let chunk_size = 1024;
    let mut samples: Vec<f32> = Default::default();
    let mut sample_buf: Vec<f32> = Default::default();
    let mut resampler: Option<Fft<f32>> = None;

    while let Some(packet) = match format.next_packet() {
      Ok(p) => p,
      Err(symphonia::core::errors::Error::IoError(e)) => {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
          None
        } else {
          return Err(AudioFileProcessingError::SymphoniaError(
            symphonia::core::errors::Error::IoError(e),
          ));
        }
      }
      Err(e) => return Err(AudioFileProcessingError::SymphoniaError(e)),
    } {
      // If the packet does not belong to the selected track, skip it.
      if packet.track_id != track_id {
        continue;
      }

      // Decode the packet into audio samples, ignoring any decode errors.
      let audio_buf = decoder.decode(&packet)?;

      // The decoded audio samples may now be accessed via the generic audio buffer
      // returned by the decoder. You may match on the buffer to access a sample-format
      // specific buffer, or use generic routines to copy out the audio samples in the
      // desired sample format.

      // Copy the decoded audio buffer into the sample buffer in an interleaved format.
      sample_buf.resize(audio_buf.samples_interleaved(), 0.);
      audio_buf.copy_to_slice_interleaved(&mut sample_buf);

      // Convert stereo samples to mono samples and write the results
      match audio_buf.spec().channels().count() {
        1 => {
          samples.extend_from_slice(&sample_buf);
        }
        2 => {
          samples.extend(
            sample_buf
              .chunks_exact(2)
              .map(|chunk| (chunk[0] + chunk[1]) * 0.5),
          );
        }
        _ => {
          return Err(AudioFileProcessingError::ReadError(
            "Only mono and stereo audio files are supported".to_string(),
          ));
        }
      }
    }

    // Resample if the file samplerate does not match the host samplerate
    if file_sample_rate != self.host_sample_rate {
      // Initialize resampler
      if resampler.is_none() {
        resampler = Some(Fft::new(
          file_sample_rate,
          self.host_sample_rate,
          chunk_size,
          2, // number of FFT blocks per processing call
          1,
          FixedSync::Both,
        )?);
      }

      let output_size = resampler
        .as_mut()
        .unwrap()
        .process_all_needed_output_len(samples.len());
      let mut resample_buffer = vec![0.; output_size];
      let input_adapter = InterleavedSlice::new(&samples, 1, samples.len())?;
      let mut output_adapter = InterleavedSlice::new_mut(&mut resample_buffer, 1, output_size)?;
      resampler.as_mut().unwrap().process_all_into_buffer(
        &input_adapter,
        &mut output_adapter,
        samples.len(),
        None,
      )?;

      samples.resize(resample_buffer.len(), 0.);
      samples.copy_from_slice(&resample_buffer);
    }

    // Calculate file duration (capped at the max buffer size)
    let duration_in_samples = if samples.len() > self.max_size {
      self.max_size
    } else {
      samples.len()
    };
    let duration_in_ms = duration_in_samples as f32 / self.host_sample_rate as f32 * 1000.;
    samples.resize(self.max_size, 0.); // Pad the buffer so it's full

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
    let audio_file_processor = AudioFileProcessor::new(44100., 44100);
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
