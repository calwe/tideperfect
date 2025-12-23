use std::time::Duration;
use std::{io::Cursor, sync::Arc};

use dash_mpd::{SegmentTimeline, MPD};
use reqwest::Client;
use ringbuf::{traits::Producer, CachingProd, HeapRb};
use stream_download::http::HttpStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use symphonia::core::{audio::{SampleBuffer}, codecs::{DecoderOptions, CODEC_TYPE_NULL}, formats::FormatOptions, 
        io::{MediaSourceStream, ReadOnlySource}, meta::MetadataOptions, probe::Hint};
use tracing::{instrument, trace};

#[instrument(skip(producer, mpd), err)]
pub async fn stream_dash_audio(mut producer: CachingProd<Arc<HeapRb<i32>>>, mpd: MPD) -> Result<(), String> {
    trace!("Streaming...");
    let client = Client::new();

    trace!("Getting seg template");
    let repr = &mpd.periods[0].adaptations[0].representations[0];
    let seg_template = repr.SegmentTemplate.as_ref().ok_or("No SegmentTemplate")?;

    trace!("Getting sample_rate");
    let sample_rate: u32 = repr.audioSamplingRate
        .as_ref()
        .and_then(|s| s.parse().ok())
        .ok_or("No SampleRate")?;
    trace!("Sample rate: {sample_rate}");

    let init_url = seg_template.initialization.as_ref().ok_or("No initialization url")?;
    trace!("initialization url: {init_url}");

    let init_data = client.get(init_url).send().await
        .map_err(|e| e.to_string())?
        .bytes().await
        .map_err(|e| e.to_string())?
        .to_vec();

    let track_info = parse_init_segment(&init_data)?;
    let channels = track_info.channels;
    trace!("Track info: {track_info:#?}");

    let timeline = seg_template.SegmentTimeline.as_ref()
        .ok_or("No SegmentTimeline")?;

    let num_segments = calculate_num_segments(timeline);
    trace!("Segment count: {num_segments}");

    for seg_num in 1..=num_segments {
        trace!("Fetching segment {seg_num}/{num_segments}");

        let seg_url = seg_template.media.as_ref()
            .ok_or("No media template")?
            .replace("$Number$", &seg_num.to_string());

        let seg_data = client.get(seg_url).send().await
            .map_err(|e| e.to_string())?
            .bytes().await
            .map_err(|e| e.to_string())?
            .to_vec();

        let mut complete_data = init_data.clone();
        complete_data.extend_from_slice(&seg_data);

        let mut samples = decode_segment(complete_data)?.into_iter().peekable();

        while samples.peek().is_some() {
            producer.push_iter(&mut samples);
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    }

    Ok(())
}

#[derive(Debug)]
struct TrackInfo {
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u32,
}

#[instrument(skip(data), err)]
fn parse_init_segment(data: &[u8]) -> Result<TrackInfo, String> {
    let cursor = Cursor::new(data.to_vec());
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let hint = Hint::new();
    // No hint - let Symphonia auto-detect the format

    trace!("Probing");
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default()).map_err(|e| e.to_string())?;
    trace!("Probed");

    let track = probed.format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No audio track")?;

    let channels = track.codec_params.channels
        .map(|c| c.count() as u16)
        .ok_or("No channels?")?;
    let sample_rate = track.codec_params.sample_rate.ok_or("No sample rate?")?;
    let bits_per_sample = track.codec_params.bits_per_sample.ok_or("No bits_per_sample")?;

    Ok(TrackInfo { channels, sample_rate, bits_per_sample })
}

#[instrument]
fn calculate_num_segments(timeline: &SegmentTimeline) -> u64 {
    let mut total = 0u64;
    for s in &timeline.segments {
        total += 1 + s.r.unwrap_or(0) as u64;
    }
    total
}

#[instrument(skip(data), err)]
fn decode_segment(data: Vec<u8>) -> Result<Vec<i32>, String> {
    let cursor = Cursor::new(data);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp4");

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default()).map_err(|e| e.to_string())?;

    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No audio track found")?;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default()).map_err(|e| e.to_string())?;

    let track_id = track.id;
    let mut sample_buf: Option<SampleBuffer<i32>> = None;
    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;

        if sample_buf.is_none() {
            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            sample_buf = Some(SampleBuffer::<i32>::new(duration, spec));
        }

        if let Some(ref mut buf) = sample_buf {
            buf.copy_interleaved_ref(decoded);
            samples.extend(buf.samples());
        }
    }

    Ok(samples)
}

#[instrument(skip(producer), err)]
pub async fn stream_url(mut producer: CachingProd<Arc<HeapRb<i32>>>, url: String) -> Result<(), String> {
    trace!("Streaming URL: {}", url);

    // Create HTTP stream with temporary file storage
    let stream = HttpStream::new(
        Client::new(),
        url.parse().map_err(|e: url::ParseError| e.to_string())?
    ).await.map_err(|e| e.to_string())?;

    // Configure streaming settings to prefetch enough data for FLAC headers
    // FLAC headers need to be fully available before Symphonia can probe
    let settings = Settings::default()
        .prefetch_bytes(2 * 1024 * 1024); // Prefetch 2MB to ensure headers are available

    trace!("Creating stream download reader");

    // Create the stream download reader - this downloads in background while we read
    let reader = StreamDownload::from_stream(
        stream,
        TempStorageProvider::default(),
        settings
    ).await.map_err(|e| e.to_string())?;

    trace!("Streaming reader created, spawning blocking decode task");

    // Run the synchronous decoding work in a blocking task to avoid blocking the async runtime
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        trace!("In blocking task, setting up decoder");

        // Wrap the reader with ReadOnlySource (which implements MediaSource)
        let media_source = ReadOnlySource::new(reader);
        let mss = MediaSourceStream::new(Box::new(media_source), Default::default());

        let mut hint = Hint::new();
        hint.with_extension("flac");

        trace!("Probing FLAC stream");
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| e.to_string())?;

        let mut format = probed.format;

        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No audio track found")?;

        trace!("Creating decoder for FLAC stream");
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| e.to_string())?;

        let track_id = track.id;
        let mut sample_buf: Option<SampleBuffer<i32>> = None;

        trace!("Decoding and streaming audio packets");
        // As packets arrive from the HTTP stream (downloaded in background),
        // decode and push them to the producer. This happens incrementally -
        // we don't wait for the full file to download
        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }

            let decoded = decoder.decode(&packet).map_err(|e| e.to_string())?;

            if sample_buf.is_none() {
                let spec = *decoded.spec();
                let duration = decoded.capacity() as u64;
                sample_buf = Some(SampleBuffer::<i32>::new(duration, spec));
            }

            if let Some(ref mut buf) = sample_buf {
                buf.copy_interleaved_ref(decoded);

                let mut samples = buf.samples().iter().copied().peekable();
                while samples.peek().is_some() {
                    producer.push_iter(&mut samples);
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }

        trace!("Finished streaming URL");
        Ok(())
    })
    .await
    .map_err(|e| format!("Blocking task failed: {}", e))??;

    Ok(())
}
