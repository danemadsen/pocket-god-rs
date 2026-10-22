//! The engine's samples as Bevy audio: FLAC decoded with claxon, the original loader's click
//! fix applied (`pg_engine::sound::apply_click_fix`), then wrapped as 16-bit PCM WAV, which
//! `bevy_audio` plays as it is.

use std::io::Cursor;

/// A decoded sample: interleaved 16-bit PCM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pcm {
    pub sample_rate: u32,
    pub channels: u16,
    pub samples: Vec<i16>,
}

/// Decodes a FLAC file to interleaved 16-bit samples (every shipped sound is 16-bit).
pub fn decode_flac(bytes: &[u8]) -> Result<Pcm, String> {
    let cursor = Cursor::new(bytes);
    let mut reader = claxon::FlacReader::new(cursor).map_err(|error| error.to_string())?;
    let info = reader.streaminfo();
    let channels = u16::try_from(info.channels).map_err(|_| "too many channels".to_owned())?;
    let bits = info.bits_per_sample;
    if bits == 0 || bits > 32 {
        return Err(format!("{bits} bits per sample"));
    }

    let mut samples = Vec::new();
    if let Some(total) = info.samples {
        let interleaved = total as usize * usize::from(channels);
        samples.reserve(interleaved);
    }
    for sample in reader.samples() {
        let sample = sample.map_err(|error| error.to_string())?;
        let sample_16 = if bits >= 16 { sample >> (bits - 16) } else { sample << (16 - bits) };
        samples.push(sample_16 as i16);
    }

    let pcm = Pcm {
        sample_rate: info.sample_rate,
        channels,
        samples,
    };
    Ok(pcm)
}

/// A RIFF / WAVE file holding `pcm` as 16-bit little-endian PCM.
pub fn encode_wav(pcm: &Pcm) -> Vec<u8> {
    const HEADER_LEN: usize = 44;
    const FMT_CHUNK_LEN: u32 = 16;
    const FORMAT_PCM: u16 = 1;
    const BITS_PER_SAMPLE: u16 = 16;

    let data_len = pcm.samples.len() * 2;
    let block_align = pcm.channels * (BITS_PER_SAMPLE / 8);
    let byte_rate = pcm.sample_rate * u32::from(block_align);
    let riff_len = (HEADER_LEN - 8 + data_len) as u32;

    let mut wav = Vec::with_capacity(HEADER_LEN + data_len);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&FMT_CHUNK_LEN.to_le_bytes());
    wav.extend_from_slice(&FORMAT_PCM.to_le_bytes());
    wav.extend_from_slice(&pcm.channels.to_le_bytes());
    wav.extend_from_slice(&pcm.sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_len as u32).to_le_bytes());
    for sample in &pcm.samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

impl Pcm {
    /// The length in seconds.
    pub fn duration(&self) -> f32 {
        let channels = usize::from(self.channels.max(1));
        let frames = self.samples.len() / channels;
        if self.sample_rate == 0 {
            return 0.0;
        }
        frames as f32 / self.sample_rate as f32
    }
}

/// A sample ready for Bevy.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayableSample {
    /// The click-fixed sample as a WAV file.
    pub wav: Vec<u8>,

    /// Its length in seconds.
    pub duration: f32,
}

/// An embedded FLAC sample as Bevy plays it: decoded, click-fixed, re-wrapped as WAV.
pub fn playable_sample(flac: &[u8]) -> Result<PlayableSample, String> {
    let mut pcm = decode_flac(flac)?;
    pg_engine::sound::apply_click_fix(&mut pcm.samples);
    let sample = PlayableSample {
        wav: encode_wav(&pcm),
        duration: pcm.duration(),
    };
    Ok(sample)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flac(path: &str) -> &'static [u8] {
        pg_assets::file(path).expect("the sample is embedded")
    }

    #[test]
    fn wav_headers_describe_the_samples() {
        let pcm = Pcm {
            sample_rate: 22_000,
            channels: 2,
            samples: vec![1, -2, 3, i16::MIN],
        };
        let wav = encode_wav(&pcm);
        assert_eq!(wav.len(), 44 + 8);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(u32::from_le_bytes([wav[4], wav[5], wav[6], wav[7]]), 36 + 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 2);
        assert_eq!(u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]), 22_000);
        assert_eq!(u32::from_le_bytes([wav[28], wav[29], wav[30], wav[31]]), 88_000);
        assert_eq!(u16::from_le_bytes([wav[32], wav[33]]), 4);
        assert_eq!(&wav[36..40], b"data");
        assert_eq!(&wav[44..], &[1, 0, 0xfe, 0xff, 3, 0, 0x00, 0x80]);
    }

    #[test]
    fn shipped_samples_decode_at_their_rate_and_channels() {
        let mono = decode_flac(flac("sounds/burp1.flac")).expect("valid FLAC");
        assert_eq!((mono.sample_rate, mono.channels), (44_100, 1));
        assert!(!mono.samples.is_empty());

        let stereo = decode_flac(flac("sounds/crunch.flac")).expect("valid FLAC");
        assert_eq!((stereo.sample_rate, stereo.channels), (22_000, 2));
        assert_eq!(stereo.samples.len() % 2, 0);
    }

    #[test]
    fn durations_count_frames_not_samples() {
        let pcm = Pcm {
            sample_rate: 22_000,
            channels: 2,
            samples: vec![0; 44_000],
        };
        assert_eq!(pcm.duration(), 1.0);

        let sample = playable_sample(flac("sounds/burp1.flac")).expect("valid FLAC");
        let mono = decode_flac(flac("sounds/burp1.flac")).expect("valid FLAC");
        let expected = mono.samples.len() as f32 / 44_100.0;
        assert!((sample.duration - expected).abs() < 1e-6);
    }

    #[test]
    fn the_click_fix_reaches_the_played_samples() {
        // One of the ten files whose last samples hold -32768 (audio-prefs-text.md §1.4.4).
        let bytes = flac("sounds/bubble3.flac");
        let raw = decode_flac(bytes).expect("valid FLAC");
        let tail_start = raw.samples.len() - 3;
        assert!(raw.samples[tail_start..].contains(&i16::MIN));

        let sample = playable_sample(bytes).expect("valid FLAC");
        let data = &sample.wav[44..];
        let (tail_pairs, _) = data[data.len() - 6..].as_chunks::<2>();
        let played_tail: Vec<i16> = tail_pairs.iter().map(|pair| i16::from_le_bytes(*pair)).collect();
        assert!(!played_tail.contains(&i16::MIN));
        assert_eq!(data.len(), raw.samples.len() * 2);
    }
}
