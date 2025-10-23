use std::{fs::File, path::Path};

use symphonia::{
    core::{
        audio::SampleBuffer,
        codecs::{CODEC_TYPE_NULL, DecoderOptions},
        errors::Error as SymphoniaError,
        formats::{FormatOptions, FormatReader},
        io::MediaSourceStream,
        meta::{MetadataOptions, StandardTagKey},
        probe::Hint,
    },
    default::{get_codecs, get_probe},
};

use crate::MainErr;

type EncodedData = Box<dyn FormatReader + 'static>;

// Requires audio files - Called by get_data
#[cfg(not(tarpaulin_include))]
pub fn parse_metadata(format: &mut EncodedData) -> (String, String) {
    let mut artist = "Unknown Artist".to_string();
    let mut title = "Unknown Title".to_string();

    if let Some(metadata) = format.metadata().current() {
        for tag in metadata.tags() {
            if let Some(std_key) = tag.std_key {
                match std_key {
                    StandardTagKey::Artist => artist = tag.value.to_string(),
                    StandardTagKey::TrackTitle => title = tag.value.to_string(),
                    _ => {}
                }
            }
        }
    }

    return (artist, title);
}

pub fn extract_title_from_filename(filename: &str) -> String {
    let title = Path::new(filename)
        .file_stem()
        .and_then(|s| return s.to_str())
        .unwrap_or(filename)
        .replace("_", " ");

    return title.replace("-", " ");
}

// Requires audio files - See ignored tests
#[cfg(not(tarpaulin_include))]
pub fn get_data(filename: String) -> Result<(String, String, EncodedData), MainErr> {
    let file = Box::new(File::open(&filename)?);
    let mss = MediaSourceStream::new(file, Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(&filename).extension().and_then(|s| return s.to_str()) {
        hint.with_extension(ext);
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let mut probed = get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
    let (artist, mut title) = parse_metadata(&mut probed.format);

    if title == *"Unknown Title" {
        title = extract_title_from_filename(&filename);
    }

    return Ok((artist, title, probed.format));
}

// Requires audio files - Audio decoding integration
#[cfg(not(tarpaulin_include))]
pub fn parse_audio_data(mut format: EncodedData) -> Result<(Vec<f32>, u32, u32), MainErr> {
    let track = format
        .tracks()
        .iter()
        .find(|t| return t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| anyhow::anyhow!("No suitable audio track found"))?;

    let track_id = track.id;

    let dec_opts = DecoderOptions { verify: true };

    let mut decoder = get_codecs().make(&track.codec_params, &dec_opts)?;

    let track_params = &track.codec_params;
    let sample_rate = track_params.sample_rate.unwrap_or(44100);
    let channels = track_params.channels.map(|c| return c.count()).unwrap_or(2) as u32;

    let mut pcm_data = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => {
                continue;
            }
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(err) => return Err(err.into()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                sample_buf.copy_interleaved_ref(decoded);
                pcm_data.extend_from_slice(sample_buf.samples());
            }
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(SymphoniaError::DecodeError(_)) => {
                continue;
            }
            Err(err) => return Err(err.into()),
        }
    }

    return Ok((pcm_data, sample_rate, channels));
}
