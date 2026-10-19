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

pub fn validate_assembly(a: &crate::contracts::Assembly, turns: &[&str]) -> Result<()> {
    if a.pause_ms > 10000
        || a.turn_gain_db.iter().any(|(id, g)| {
            !turns.contains(&id.as_str()) || !g.is_finite() || !(-24.0..=24.0).contains(g)
        })
        || a.loudness_lufs
            .is_some_and(|v| !v.is_finite() || !(-36.0..=-10.0).contains(&v))
    {
        return Err(Error::input(
            "Assembly requires pause_ms 0–10000, known turn IDs, gains -24–24 dB, and loudness_lufs -36–-10",
        ));
    }
    Ok(())
}
/// Native mono assembly. Global gain is constrained by a -1 dBFS sample-peak ceiling.
pub fn assemble(
    clips: &[(String, Vec<f32>)],
    options: Option<&crate::contracts::Assembly>,
) -> Result<Vec<f32>> {
    if let Some(a) = options {
        validate_assembly(a, &clips.iter().map(|c| c.0.as_str()).collect::<Vec<_>>())?;
    }
    let mut out = Vec::new();
    for (i, (id, clip)) in clips.iter().enumerate() {
        if clip.iter().any(|s| !s.is_finite()) {
            return Err(Error::input("Nonfinite audio samples"));
        }
        if i > 0
            && let Some(a) = options
        {
            out.resize(out.len() + a.pause_ms as usize * 24, 0.0);
        }
        let gain = options
            .and_then(|a| a.turn_gain_db.get(id))
            .map(|db| 10_f32.powf(db / 20.0))
            .unwrap_or(1.0);
        out.extend(clip.iter().map(|v| v * gain));
    }
    if let Some(a) = options {
        let peak = out.iter().map(|v| v.abs()).fold(0.0_f32, f32::max);
        let mut gain = 1.0;
        if let Some(target) = a.loudness_lufs {
            let fail = |_| Error::new("loudness", "Cannot measure integrated audio loudness", 3);
            let mut meter = ebur128::EbuR128::new(1, 24000, ebur128::Mode::I).map_err(fail)?;
            meter.add_frames_f32(&out).map_err(fail)?;
            let measured = meter.loudness_global().map_err(fail)?;
            if measured.is_finite() {
                gain = 10_f32.powf(((target - measured) / 20.0) as f32);
            }
        }
        if peak > 0.0 {
            gain = gain.min(10_f32.powf(-1.0 / 20.0) / peak);
        }
        for sample in &mut out {
            *sample *= gain;
        }
    }
    Ok(out)
}

/// Working rate for speech assembly and every published audiobook file.
pub const RATE: u32 = 24000;
const WAV_HEADER: u64 = 44;

fn sample(value: f32) -> i16 {
    // Identical to `wav`, so streamed and in-memory assembly agree bit for bit.
    (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16
}
/// Largest mono 16-bit sample count a RIFF/WAV file can hold (about 24.8 hours at 24 kHz).
pub fn wav_capacity() -> u64 {
    (u32::MAX as u64 - WAV_HEADER) / 2
}
fn write_error(message: &str) -> Error {
    Error::new("audio_write", message, 3)
}

#[cfg(feature = "mp3")]
struct Mp3Out {
    encoder: mp3lame_encoder::Encoder,
    file: std::io::BufWriter<std::fs::File>,
    buffer: Vec<u8>,
}
#[cfg(feature = "mp3")]
fn bitrate(kbps: u16) -> Result<mp3lame_encoder::Bitrate> {
    use mp3lame_encoder::Bitrate::*;
    Ok(match kbps {
        8 => Kbps8,
        16 => Kbps16,
        24 => Kbps24,
        32 => Kbps32,
        40 => Kbps40,
        48 => Kbps48,
        64 => Kbps64,
        80 => Kbps80,
        96 => Kbps96,
        112 => Kbps112,
        128 => Kbps128,
        160 => Kbps160,
        _ => return Err(Error::input("Unsupported MP3 bitrate")),
    })
}

/// Streaming 24 kHz mono sink writing WAV, MP3 (CBR), or both from one pass.
pub struct Writer {
    wav: Option<hound::WavWriter<std::io::BufWriter<std::fs::File>>>,
    wav_file: Option<std::fs::File>,
    #[cfg(feature = "mp3")]
    mp3: Option<Mp3Out>,
    samples: u64,
    pcm: Vec<i16>,
}
impl Writer {
    /// `mp3` is (file, CBR kbps, ID3v2 tag bytes written before the first frame).
    pub fn new(
        wav: Option<std::fs::File>,
        mp3: Option<(std::fs::File, u16, Vec<u8>)>,
    ) -> Result<Self> {
        let wav_file = wav.as_ref().map(|f| f.try_clone()).transpose()?;
        let wav = wav
            .map(|file| {
                hound::WavWriter::new(
                    std::io::BufWriter::new(file),
                    hound::WavSpec {
                        channels: 1,
                        sample_rate: RATE,
                        bits_per_sample: 16,
                        sample_format: hound::SampleFormat::Int,
                    },
                )
                .map_err(|_| write_error("Cannot create WAV"))
            })
            .transpose()?;
        #[cfg(feature = "mp3")]
        let mp3 = mp3
            .map(|(file, kbps, tag)| -> Result<Mp3Out> {
                use std::io::Write;
                let mut builder = mp3lame_encoder::Builder::new()
                    .ok_or_else(|| write_error("Cannot start the MP3 encoder"))?;
                let fail = |_| write_error("Cannot configure the MP3 encoder");
                builder.set_num_channels(1).map_err(fail)?;
                builder.set_sample_rate(RATE).map_err(fail)?;
                builder.set_brate(bitrate(kbps)?).map_err(fail)?;
                builder
                    .set_mode(mp3lame_encoder::Mode::Mono)
                    .map_err(fail)?;
                builder
                    .set_quality(mp3lame_encoder::Quality::Good)
                    .map_err(fail)?;
                builder.set_to_write_vbr_tag(false).map_err(fail)?;
                let encoder = builder.build().map_err(fail)?;
                let mut file = std::io::BufWriter::new(file);
                file.write_all(&tag)?;
                Ok(Mp3Out {
                    encoder,
                    file,
                    buffer: vec![],
                })
            })
            .transpose()?;
        #[cfg(not(feature = "mp3"))]
        if mp3.is_some() {
            return Err(Error::input("This build has no MP3 encoder"));
        }
        Ok(Self {
            wav,
            wav_file,
            #[cfg(feature = "mp3")]
            mp3,
            samples: 0,
            pcm: vec![],
        })
    }
    pub fn write(&mut self, samples: &[f32]) -> Result<()> {
        self.samples += samples.len() as u64;
        if self.wav.is_some() && self.samples > wav_capacity() {
            return Err(Error::input(
                "Audio exceeds the WAV size limit (about 24.8 hours); use audiobook.format: mp3 or output_mode: chapters",
            ));
        }
        self.pcm.clear();
        self.pcm.extend(samples.iter().map(|s| sample(*s)));
        if let Some(wav) = &mut self.wav {
            let mut writer = wav.get_i16_writer(self.pcm.len() as u32);
            for s in &self.pcm {
                writer.write_sample(*s);
            }
            writer
                .flush()
                .map_err(|_| write_error("Cannot write WAV samples"))?;
        }
        #[cfg(feature = "mp3")]
        if let Some(out) = &mut self.mp3 {
            use std::io::Write;
            for block in self.pcm.chunks(8192) {
                out.buffer.clear();
                out.buffer
                    .reserve(mp3lame_encoder::max_required_buffer_size(block.len()));
                out.encoder
                    .encode_to_vec(mp3lame_encoder::MonoPcm(block), &mut out.buffer)
                    .map_err(|_| write_error("MP3 encoding failed"))?;
                out.file.write_all(&out.buffer)?;
            }
        }
        Ok(())
    }
    pub fn silence(&mut self, samples: usize) -> Result<()> {
        let block = vec![0.0; 4096];
        let mut left = samples;
        while left > 0 {
            let n = left.min(block.len());
            self.write(&block[..n])?;
            left -= n;
        }
        Ok(())
    }
    /// Finalize every output and return the number of samples written.
    pub fn finish(self) -> Result<u64> {
        if let Some(wav) = self.wav {
            wav.finalize()
                .map_err(|_| write_error("Cannot finalize WAV"))?;
        }
        if let Some(file) = self.wav_file {
            file.sync_all()?;
        }
        #[cfg(feature = "mp3")]
        if let Some(mut out) = self.mp3 {
            use std::io::Write;
            out.buffer.clear();
            out.buffer.reserve(7200);
            out.encoder
                .flush_to_vec::<mp3lame_encoder::FlushGap>(&mut out.buffer)
                .map_err(|_| write_error("Cannot finish MP3 encoding"))?;
            out.file.write_all(&out.buffer)?;
            out.file.flush()?;
            out.file
                .into_inner()
                .map_err(|_| write_error("Cannot finalize MP3"))?
                .sync_all()?;
        }
        Ok(self.samples)
    }
}

/// Stream 24 kHz mono samples from an audio file; other formats are decoded and resampled.
pub fn stream(path: &std::path::Path, mut f: impl FnMut(&[f32]) -> Result<()>) -> Result<u64> {
    if let Ok(mut reader) = hound::WavReader::open(path) {
        let spec = reader.spec();
        if spec.channels == 1
            && spec.sample_rate == RATE
            && spec.bits_per_sample == 16
            && spec.sample_format == hound::SampleFormat::Int
        {
            let mut block = Vec::with_capacity(8192);
            let mut total = 0;
            for s in reader.samples::<i16>() {
                let s =
                    s.map_err(|_| Error::new("invalid_audio", "Audio samples are corrupt", 3))?;
                block.push(s as f32 / i16::MAX as f32);
                if block.len() == 8192 {
                    total += block.len() as u64;
                    f(&block)?;
                    block.clear();
                }
            }
            total += block.len() as u64;
            if !block.is_empty() {
                f(&block)?;
            }
            return Ok(total);
        }
    }
    let samples = mono_24k(&std::fs::read(path)?)?;
    f(&samples)?;
    Ok(samples.len() as u64)
}

/// Duration in seconds without keeping decoded samples in memory.
pub fn probe(path: &std::path::Path) -> Result<f64> {
    if let Ok(reader) = hound::WavReader::open(path) {
        let spec = reader.spec();
        return Ok(reader.duration() as f64 / spec.sample_rate as f64);
    }
    let file = std::fs::File::open(path)?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());
    let invalid = || Error::new("invalid_audio", "Cannot decode audio file", 3);
    let mut format = symphonia::default::get_probe()
        .format(
            &Hint::new(),
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| invalid())?
        .format;
    let track = format.default_track().ok_or_else(invalid)?;
    let id = track.id;
    let rate = track.codec_params.sample_rate.ok_or_else(invalid)?;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| invalid())?;
    let mut frames = 0u64;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => return Err(invalid()),
        };
        if packet.track_id() != id {
            continue;
        }
        frames += decoder.decode(&packet).map_err(|_| invalid())?.frames() as u64;
    }
    if frames == 0 {
        return Err(Error::new("invalid_audio", "Audio contains no samples", 3));
    }
    Ok(frames as f64 / rate as f64)
}

/// Two-pass streaming assembly: measure loudness and peak, then write with one gain.
/// Matches `assemble` (pauses, loudness target, -1 dBFS peak ceiling) with bounded memory.
pub fn assemble_files(
    clips: &[std::path::PathBuf],
    pause_ms: u32,
    loudness_lufs: Option<f64>,
    writer: &mut Writer,
    cancelled: &dyn Fn() -> bool,
) -> Result<()> {
    let fail = |_| Error::new("loudness", "Cannot measure integrated audio loudness", 3);
    let mut meter = ebur128::EbuR128::new(1, RATE, ebur128::Mode::I).map_err(fail)?;
    let mut peak = 0.0_f32;
    let pause = vec![0.0_f32; pause_ms as usize * 24];
    for (i, clip) in clips.iter().enumerate() {
        if i > 0 && !pause.is_empty() {
            meter.add_frames_f32(&pause).map_err(fail)?;
        }
        stream(clip, |block| {
            if block.iter().any(|s| !s.is_finite()) {
                return Err(Error::input("Nonfinite audio samples"));
            }
            peak = block.iter().map(|v| v.abs()).fold(peak, f32::max);
            meter.add_frames_f32(block).map_err(fail)
        })?;
    }
    let mut gain = 1.0_f32;
    if let Some(target) = loudness_lufs {
        let measured = meter.loudness_global().map_err(fail)?;
        if measured.is_finite() {
            gain = 10_f32.powf(((target - measured) / 20.0) as f32);
        }
    }
    if peak > 0.0 {
        gain = gain.min(10_f32.powf(-1.0 / 20.0) / peak);
    }
    let mut scaled = vec![];
    for (i, clip) in clips.iter().enumerate() {
        if cancelled() {
            return Err(Error::new("cancelled", "Assembly cancelled", 3));
        }
        if i > 0 {
            writer.silence(pause.len())?;
        }
        stream(clip, |block| {
            scaled.clear();
            scaled.extend(block.iter().map(|v| v * gain));
            writer.write(&scaled)
        })?;
    }
    Ok(())
}

/// Concatenate already-levelled clips with pauses; returns each clip's sample count.
pub fn concat_files(
    clips: &[std::path::PathBuf],
    pause_ms: u32,
    writer: &mut Writer,
    cancelled: &dyn Fn() -> bool,
) -> Result<Vec<u64>> {
    let mut lengths = vec![];
    for (i, clip) in clips.iter().enumerate() {
        if cancelled() {
            return Err(Error::new("cancelled", "Assembly cancelled", 3));
        }
        if i > 0 {
            writer.silence(pause_ms as usize * 24)?;
        }
        lengths.push(stream(clip, |block| writer.write(block))?);
    }
    Ok(lengths)
}
