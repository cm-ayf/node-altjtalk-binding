use jbonsai::{engine::Condition, speech::SpeechGenerator};

use super::{pcm::PcmEncoder, Application, Channels, Encoder, EncoderConfig, EncoderType};

pub struct OpusEncoder {
  pcm_encoder: PcmEncoder,
  opus_encoder: audiopus::coder::Encoder,
}

const OPUS_FRAME_SIZE: usize = 20; // ms

impl Encoder for OpusEncoder {
  fn new(condition: &Condition, config: &EncoderConfig) -> napi::Result<Self>
  where
    Self: Sized,
  {
    let sample_rate = audiopus::SampleRate::try_from(condition.get_sampling_frequency() as i32)
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;
    let channels = config.channels.unwrap_or(Channels::Stereo);
    let mode = config.mode.unwrap_or(Application::Voip);

    let synthesis_per_second = condition.get_sampling_frequency() / condition.get_fperiod();
    let chunk_size = synthesis_per_second * OPUS_FRAME_SIZE / 1000;
    let pcm_config = EncoderConfig {
      r#type: EncoderType::Raw,
      channels: Some(channels),
      mode: None,
      chunk_size: Some(chunk_size as u32),
    };

    let pcm_encoder = PcmEncoder::new(condition, &pcm_config)?;
    let opus_encoder = audiopus::coder::Encoder::new(sample_rate, channels.into(), mode.into())
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;

    Ok(Self {
      pcm_encoder,
      opus_encoder,
    })
  }

  fn generate(&mut self, generator: &mut SpeechGenerator) -> napi::Result<Vec<u8>> {
    let pcm: Vec<_> = self.pcm_encoder.generate_i16(generator).collect();
    let mut output = vec![0; self.pcm_encoder.speech_len()];

    let size = self
      .opus_encoder
      .encode(&pcm, &mut output)
      .map_err(|err| napi::Error::new(napi::Status::GenericFailure, err))?;
    output.resize(size, 0);

    Ok(output)
  }
}