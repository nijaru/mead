# mead

**M**emory-safe **E**ncoding **A**nd **D**ecoding

A modern, safe media processing toolkit written in Rust, designed to prevent the memory safety vulnerabilities that plague traditional media libraries.

## Goals

- **Memory Safety First**: Pure Rust implementations and safe bindings to eliminate buffer overflows, use-after-free, and other memory safety issues
- **Modern Codecs**: Focus on widely-used formats (AV1, H.264, AAC, Opus) rather than legacy codec sprawl
- **Clean Architecture**: Modular design with clear separation between containers, codecs, and processing pipeline
- **Production Ready**: Comprehensive error handling, logging, and fuzzing from day one

## Project Status

🚧 **Early Development** - Not ready for production use

Currently implementing Phase 1: MP4 container support + AV1 codec

## Architecture

```
mead-cli/          # Command-line interface
mead-core/         # Core library
  ├── container/   # Format handlers (MP4, WebM, MKV)
  ├── codec/       # Codec implementations (AV1, H.264, AAC)
  ├── pipeline/    # Processing pipeline
  └── io/          # I/O abstraction
```

## Roadmap

- **Phase 1** (Current): MP4 container + AV1 codec
- **Phase 2**: Audio support (AAC, Opus)
- **Phase 3**: H.264, H.265, VP9 codecs
- **Phase 4**: WebM/MKV containers
- **Phase 5**: Streaming protocols (HLS, DASH, RTMP)

## Safety Approach

- Pure Rust libraries preferred: `rav1e`, `rav1d`, `mp4parse-rust`, `symphonia`
- Safe bindings for mature C libraries where necessary
- `#![forbid(unsafe_code)]` in safe modules
- Fuzzing integrated in CI from day one
- No unsafe FFmpeg wrappers

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! This project prioritizes code quality, safety, and correctness over speed of development.
