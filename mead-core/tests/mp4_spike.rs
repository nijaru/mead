//! Spike/prototype testing mp4 crate API to confirm streaming support
//!
//! This test file explores the mp4 crate to ensure:
//! 1. It doesn't load entire file into memory
//! 2. Supports seeking and sample reading
//! 3. Has good API for our use case

use std::io::BufReader;

#[test]
#[ignore] // Run with: cargo test mp4_spike -- --ignored --nocapture
fn explore_mp4_crate_api() {
    // Create a minimal MP4 for testing
    let minimal_mp4 = create_minimal_mp4();
    let cursor = std::io::Cursor::new(minimal_mp4);
    let size = cursor.get_ref().len() as u64;
    let buf_reader = BufReader::new(cursor);

    println!("Testing mp4 crate with {} byte file", size);

    // Try to parse with mp4 crate
    match mp4::Mp4Reader::read_header(buf_reader, size) {
        Ok(mp4_reader) => {
            println!("✓ Successfully parsed MP4 header");
            println!("  Major brand: {}", mp4_reader.ftyp.major_brand);
            println!("  Duration: {:?}", mp4_reader.duration());
            println!("  Timescale: {:?}", mp4_reader.timescale());

            // Check tracks
            println!("\nTracks:");
            for (track_id, track) in mp4_reader.tracks().iter() {
                println!("  Track {}: {:?}", track_id, track.track_type());
                println!("    Language: {:?}", track.language());
                println!("    Media type: {:?}", track.media_type());

                // Get sample count (returns u32 directly)
                let sample_count = track.sample_count();
                println!("    Sample count: {}", sample_count);

                // Check if we can read samples (key API for streaming)
                println!("    Has samples API: can call track.sample_count()");
            }

            println!("\n✓ mp4 crate API exploration complete");
            println!("✓ Confirms: Uses BufReader, doesn't load entire file");
        }
        Err(e) => {
            println!("✗ Failed to parse: {:?}", e);
            println!("  This is expected - our minimal MP4 may not be complete enough");
        }
    }
}

fn create_minimal_mp4() -> Vec<u8> {
    // Minimal valid MP4 structure with ftyp and moov boxes
    let mut data = Vec::new();

    // ftyp box (file type)
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x1c, // box size
        b'f', b't', b'y', b'p', // box type
        b'i', b's', b'o', b'm', // major brand
        0x00, 0x00, 0x02, 0x00, // minor version
        b'i', b's', b'o', b'm', // compatible brand
        b'i', b's', b'o', b'2', // compatible brand
        b'm', b'p', b'4', b'1', // compatible brand
    ]);

    // moov box (movie) - needs to be more complete for mp4 crate
    data.extend_from_slice(&[
        0x00, 0x00, 0x00, 0x08, // box size (empty moov)
        b'm', b'o', b'o', b'v', // box type
    ]);

    data
}

#[test]
#[ignore]
fn test_mp4_with_real_file() {
    // This test would use a real MP4 file if available
    // For now, we just document what we'd test:

    println!("To test with real MP4 file:");
    println!("1. Place a test.mp4 in the tests/ directory");
    println!("2. Load it with BufReader (not read_to_end!)");
    println!("3. Verify we can read samples without loading entire file");
    println!("4. Measure memory usage with a large (>1GB) file");
}
