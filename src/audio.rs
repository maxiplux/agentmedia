use crate::error::{Error, Result};
use rubato::{FftFixedInOut, Resampler};
use std::io::Cursor;
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, formats::FormatOptions, io::MediaSourceStream,
    meta::MetadataOptions, probe::Hint,
};

pub fn decode(bytes: &[u8]) -> Result<(Vec<f32>, u32)> {
    let stream = MediaSourceStream::new(Box::new(Cursor::new(bytes.to_vec())), Default::default());
    let mut format = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| {
            Error::new(
                "invalid_audio",
                "Cannot decode provider audio (expected WAV, MP3, or FLAC)",
                3,
            )
        })?
        .format;
    let track = format
        .default_track()
        .ok_or_else(|| Error::new("invalid_audio", "Audio has no track", 3))?;
    let id = track.id;
    let rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| Error::new("invalid_audio", "Audio has no sample rate", 3))?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| Error::new("invalid_audio", "Unsupported audio codec", 3))?;
    let mut samples = vec![];
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => {
                return Err(Error::new(
                    "invalid_audio",
                    "Audio packet stream is corrupt",
                    3,
                ));
            }
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = decoder
            .decode(&packet)
            .map_err(|_| Error::new("invalid_audio", "Audio samples are corrupt", 3))?;
        let channels = decoded.spec().channels.count();
        let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buf.copy_interleaved_ref(decoded);
        samples.extend(
            buf.samples()
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32),
        );
    }
    if samples.is_empty() {
        return Err(Error::new("invalid_audio", "Audio contains no samples", 3));
    }
    Ok((samples, rate))
}
pub fn mono_24k(bytes: &[u8]) -> Result<Vec<f32>> {
    let (samples, rate) = decode(bytes)?;
    if rate == 24000 {
        return Ok(samples);
    }
    let mut resampler = FftFixedInOut::<f32>::new(rate as usize, 24000, 1024, 1)
        .map_err(|_| Error::new("resample", "Cannot configure audio resampler", 3))?;
    let delay = resampler.output_delay();
    let length = (samples.len() as u64 * 24000 / rate as u64) as usize;
    let chunk = resampler.input_frames_next();
    let mut out = vec![];
    for slice in samples.chunks(chunk) {
        let mut padded = slice.to_vec();
        padded.resize(chunk, 0.0);
        let result = resampler
            .process(&[padded], None)
            .map_err(|_| Error::new("resample", "Audio resampling failed", 3))?;
        out.extend_from_slice(&result[0]);
    }
    while out.len() < length + delay {
        let result = resampler
            .process(&[vec![0.0; chunk]], None)
            .map_err(|_| Error::new("resample", "Audio resampling failed", 3))?;
        out.extend_from_slice(&result[0]);
    }
    Ok(out[delay..delay + length].to_vec())
}
pub fn wav(samples: &[f32]) -> Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 24000,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::new(&mut cursor, spec)
            .map_err(|_| Error::new("audio_write", "Cannot create WAV", 3))?;
        for s in samples {
            writer
                .write_sample((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .map_err(|_| Error::new("audio_write", "Cannot write WAV samples", 3))?;
        }
        writer
            .finalize()
            .map_err(|_| Error::new("audio_write", "Cannot finalize WAV", 3))?;
    }
    Ok(cursor.into_inner())
}
