//! FFprobe JSON to nest-media mapping.

use nest_media::{
    AudioTrack, HdrFormat, MediaInspection, MediaTracks, SubtitleTrack, VideoTrack,
};

use crate::dto::FfprobeOutput;
use crate::dto::stream::StreamSection;

/// Maps FFprobe output to [`MediaInspection`].
pub fn to_inspection(output: &FfprobeOutput) -> MediaInspection {
    let mut tracks = MediaTracks::new();
    let mut primary_video_set = false;

    for stream in &output.streams {
        match stream.codec_type.as_deref() {
            Some("video") if !primary_video_set => {
                tracks.video.push(map_video_track(stream));
                primary_video_set = true;
            }
            Some("audio") => tracks.audio.push(map_audio_track(stream)),
            Some("subtitle") => tracks.subtitles.push(map_subtitle_track(stream)),
            _ => {}
        }
    }

    let mut inspection = MediaInspection::new(tracks);

    if let Some(format) = &output.format {
        inspection.container = format
            .format_name
            .as_ref()
            .map(|name| name.split(',').next().unwrap_or(name).to_string());
        inspection.duration_seconds = format
            .duration
            .as_ref()
            .and_then(|value| value.parse::<f64>().ok())
            .map(|seconds| seconds.round() as u32);
    }

    inspection
}

fn map_video_track(stream: &StreamSection) -> VideoTrack {
    VideoTrack {
        codec: stream.codec_name.clone(),
        width: stream.width,
        height: stream.height,
        bitrate: stream.bit_rate.as_ref().and_then(|value| value.parse().ok()),
        hdr: detect_hdr(stream),
    }
}

fn map_audio_track(stream: &StreamSection) -> AudioTrack {
    let channels = stream
        .channel_layout
        .clone()
        .or_else(|| stream.channels.map(|count| count.to_string()));

    AudioTrack {
        codec: stream.codec_name.clone(),
        channels,
        language: tag_value(stream, "language"),
        title: tag_value(stream, "title"),
    }
}

fn map_subtitle_track(stream: &StreamSection) -> SubtitleTrack {
    let disposition = stream.disposition.as_ref();
    SubtitleTrack {
        codec: stream.codec_name.clone(),
        language: tag_value(stream, "language"),
        title: tag_value(stream, "title"),
        forced: disposition.is_some_and(|d| d.forced != 0),
        is_default: disposition.is_some_and(|d| d.default != 0),
    }
}

fn tag_value(stream: &StreamSection, key: &str) -> Option<String> {
    stream
        .tags
        .get(key)
        .filter(|value| !value.is_empty())
        .cloned()
}

fn detect_hdr(stream: &StreamSection) -> Option<HdrFormat> {
    for side_data in &stream.side_data_list {
        let side_data_type = side_data
            .side_data_type
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase();

        if side_data_type.contains("dolby") {
            return Some(HdrFormat::DolbyVision);
        }
        if side_data_type.contains("hlg") {
            return Some(HdrFormat::Hlg);
        }
        if side_data_type.contains("mastering display metadata")
            || side_data_type.contains("content light level")
            || side_data_type.contains("hdr")
        {
            return Some(HdrFormat::Hdr10);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::format::FormatSection;

    fn sample_output() -> FfprobeOutput {
        serde_json::from_str(include_str!("tests/fixtures/sample_ffprobe.json")).unwrap()
    }

    #[test]
    fn maps_container_and_duration() {
        let inspection = to_inspection(&sample_output());
        assert_eq!(inspection.container.as_deref(), Some("matroska"));
        assert_eq!(inspection.duration_seconds, Some(7027));
    }

    #[test]
    fn maps_video_audio_and_subtitle_tracks() {
        let inspection = to_inspection(&sample_output());
        assert_eq!(inspection.tracks.video.len(), 1);
        assert_eq!(inspection.tracks.video[0].codec.as_deref(), Some("h264"));
        assert_eq!(inspection.tracks.video[0].width, Some(1920));
        assert_eq!(inspection.tracks.audio.len(), 2);
        assert_eq!(inspection.tracks.audio[0].language.as_deref(), Some("eng"));
        assert_eq!(inspection.tracks.subtitles.len(), 1);
        assert!(inspection.tracks.subtitles[0].forced);
    }

    #[test]
    fn maps_hdr10_from_side_data() {
        let output = FfprobeOutput {
            streams: vec![StreamSection {
                codec_type: Some("video".into()),
                codec_name: Some("hevc".into()),
                width: Some(3840),
                height: Some(2160),
                bit_rate: None,
                channels: None,
                channel_layout: None,
                tags: Default::default(),
                disposition: None,
                side_data_list: vec![crate::dto::stream::SideDataSection {
                    side_data_type: Some("Mastering display metadata".into()),
                }],
            }],
            format: Some(FormatSection {
                format_name: Some("mp4".into()),
                duration: Some("3600.0".into()),
            }),
        };

        let inspection = to_inspection(&output);
        assert_eq!(inspection.tracks.video[0].hdr, Some(HdrFormat::Hdr10));
    }
}
