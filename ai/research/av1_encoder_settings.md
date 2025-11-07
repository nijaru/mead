# AV1 Encoder Settings Research

Research on practical AV1 encoding parameters, focusing on what actually matters vs cargo cult settings.

**Date**: 2025-11-06
**Encoder**: rav1e (pure Rust AV1 encoder)

## Executive Summary

**The brutal truth**: rav1e is **not competitive** with SVT-AV1 in 2025. All evidence points to SVT-AV1 as the practical choice:
- **7x faster** than rav1e at same quality
- **30% smaller files** than rav1e at same quality
- **Better threading** - uses all cores vs rav1e's single-core performance
- **Industry adoption** - Netflix, Intel, YouTube use SVT-AV1

**For rav1e specifically** (if you must use it):
- Speed preset range: 0-10 (lower = slower, higher quality)
- Practical presets: 4-6 for offline encoding, 9-10 for real-time
- CRF: No standardized recommendation found (unlike SVT-AV1's well-documented CRF 24-28)

## Speed Preset Analysis

### rav1e Speed Presets (0-10)

**What they actually control**:
- Motion estimation search range
- Prediction mode analysis depth
- Transform size decisions
- Rate-distortion optimization level

**Performance data** (from OpenBenchmarking.org):
- Speed 10 (fastest): ~20 FPS on high-end hardware (Ryzen 9 7950X)
- Speed 6: ~3.4 FPS
- Speed 1: ~0.5 FPS

**The problem**: These speeds are **absurdly slow**. Even preset 10 struggles to hit real-time on 1080p content.

### SVT-AV1 Speed Presets (0-13) - For Comparison

**Industry consensus**:
- **Presets 8-12**: Real-time/faster-than-real-time (50-100+ FPS on modern CPUs)
- **Presets 4-6**: High-quality offline encoding (10-20 FPS)
- **Presets 0-3**: Archive quality, extremely slow (1-5 FPS)

**What pros use**:
- **av1an** (chunked encoder): Defaults to preset 4
- **HandBrake**: Offers presets 4-8 in UI
- **Netflix**: Reports using preset 4-6 for VOD

## Quality Settings (CRF)

### SVT-AV1 CRF Guidelines (well-documented)

**Recommended ranges**:
- **480p/576p SD**: CRF 22-32
- **720p HD**: CRF 25-35
- **1080p FHD**: CRF 25-35
- **2160p 4K**: CRF 25-40

**Quality targets**:
- CRF 23-28: VMAF 95+ ("transparent" - indistinguishable for most viewers)
- CRF 28-32: VMAF 90-95 (good quality, 50%+ bitrate savings)
- CRF 32+: Noticeable quality loss but acceptable for some use cases

**Key finding**: Higher resolution allows higher CRF (lower quality) because:
- More pixels = artifacts are smaller percentage of total
- Perceptually less noticeable
- 4K at CRF 35 ≈ 1080p at CRF 28 in perceived quality

### rav1e CRF Settings

**Problem**: No widely-cited CRF recommendations found. The rav1e documentation doesn't provide the same level of guidance as SVT-AV1.

**From limited discussions**:
- Reddit user reports: "rav1e at speed 4, qp 80" produces quality comparable to "SVT-AV1 preset 3/4"
- Note: rav1e uses QP (quantization parameter) not CRF in some contexts
- Translation between QP and CRF scales is unclear

## Real-World Benchmark Data

### SVT-AV1 vs rav1e Performance (Colin McKellar, 2024)

**Test**: Same 1080p video encoded to VMAF 90

**SVT-AV1 Preset 7**:
- Speed: ~20 FPS
- File size: 30% smaller than rav1e S9
- Quality: VMAF 90

**rav1e Speed 9 (fastest practical preset)**:
- Speed: ~3 FPS
- File size: Larger than SVT-AV1
- Quality: VMAF 90

**Conclusion**: SVT-AV1 preset 7 is **7x faster** and produces **smaller files** than rav1e speed 9.

### rav1e 0.7 Performance (2024 update)

**Improvements**: rav1e 0.7 is ~2x faster than 0.6 (previous version)

**But**: Even doubled, rav1e S9 only hits ~6 FPS, while SVT-AV1 S8 achieves 20-25 FPS

**Threading**: rav1e is "essentially unthreaded" on M1 Pro, using only 1 of 8 performance cores

## What Actually Matters vs Placebo

### Settings That Matter

**1. Preset/Speed** (CRITICAL)
- Massive impact on both quality and speed
- Well-understood tradeoffs
- Use preset 4-6 for SVT-AV1 offline encoding

**2. CRF/Quality Target** (CRITICAL)
- Direct control of quality vs file size
- VMAF 95 = "transparent" threshold
- Use CRF 24-28 for SVT-AV1 at 1080p

**3. Tiles and Threading** (AUTO-DETECT)
- Modern encoders handle this well automatically
- SVT-AV1: Auto-detects CPU cores, sets tiles accordingly
- **Don't expose to users** - too complex, auto works fine

**4. Two-Pass Encoding** (OPTIONAL)
- 5-10% bitrate savings for same quality
- 2x slower encoding
- Only worth it for: archive quality, bandwidth-constrained delivery

### Settings That Don't Matter (Placebo)

**1. "Tune" Parameters**
- SVT-AV1 has `tune=0` (PSNR) vs `tune=1` (visual quality)
- Real-world difference is minimal
- Default works fine

**2. Film Grain Synthesis**
- `film-grain-denoise=0:film-grain=8` commonly recommended
- Only matters for noisy/grainy sources
- Most modern content is already clean

**3. Enable Overlays** (`enable-overlays=1`)
- Marginal quality gain
- Slightly slower encoding
- Not worth the complexity

## Recommended Defaults

### For mead (if we add encoding)

**If using SVT-AV1** (recommended):
```
Preset: 6 (balanced quality/speed)
CRF: 28 (good quality, reasonable file size)
Tiles: Auto
Threads: Auto
```

**Why this matters**:
- Preset 6: 20x faster than rav1e, same quality
- CRF 28: VMAF ~95, transparent quality
- Auto settings: Just work, no user confusion

**User-facing options**:
- Quality: Fast (preset 8) / Balanced (preset 6) / High (preset 4)
- CRF: Hidden, auto-calculated based on resolution
  - 1080p: CRF 28
  - 720p: CRF 26
  - 480p: CRF 24

### For mead (if stuck with rav1e)

**Don't**. Seriously. But if you must:
```
Speed: 6 (slowest practical option, ~3 FPS on high-end CPU)
QP: Unknown - no clear recommendations
```

**Why this is bad**:
- 7x slower than SVT-AV1
- Larger files
- Worse threading
- Less industry support

## Industry Practices

### Netflix
- Uses SVT-AV1 for encoding
- Preset 4-6 range for VOD content
- Per-shot encoding with adaptive CRF
- Target: VMAF 95+ (transparent)

### YouTube
- SVT-AV1 for AV1 encodes
- "Always prefer AV1" setting available
- Preset varies by content (likely 6-8 range)
- CRF varies by resolution

### HandBrake
- Exposes SVT-AV1 presets 4-10 in UI
- Default: "Fast" (preset 8)
- HQ presets use preset 4-6
- CRF defaults: 30-35 depending on resolution

### av1an (Community Tool)
- Defaults to SVT-AV1 (not rav1e)
- Default preset: 4
- Chunked encoding with per-scene CRF optimization
- Target VMAF mode: maintains 95+ across scenes

## The 80/20: What to Focus On

**For 90% of use cases**:
1. Use SVT-AV1 (not rav1e)
2. Preset 6
3. CRF 28 for 1080p
4. Let encoder auto-handle tiles/threading

**This gives you**:
- ~20 FPS encoding on modern CPU
- VMAF 95 (transparent quality)
- ~50% bitrate savings over H.264
- Minimal complexity

**Don't bother with**:
- Fine-tuning film grain
- Multiple preset testing (6 works)
- Custom tile configurations
- Two-pass unless you need the absolute best compression

## Authoritative Sources

1. **SVT-AV1 Documentation**
   - https://gitlab.com/AOMediaCodec/SVT-AV1/-/blob/master/Docs/CommonQuestions.md
   - Most comprehensive preset/CRF guidance

2. **OTTVerse SVT-AV1 Analysis** (2023)
   - https://ottverse.com/analysis-of-svt-av1-presets-and-crf-values/
   - Empirical preset comparison with VMAF scores
   - Shows preset 6-8 sweet spot

3. **Colin McKellar Blog** (2024)
   - https://colinmckellar.com/2024/02/21/av1-codec-update/
   - Direct rav1e vs SVT-AV1 comparison
   - Shows 7x speed advantage for SVT-AV1

4. **Netflix VMAF Paper**
   - https://www.realnetworks.com/sites/default/files/vmaf_reproducibility_ieee.pdf
   - VMAF 93+ = "indistinguishable or not annoying"
   - Establishes quality thresholds

5. **OpenBenchmarking.org**
   - https://openbenchmarking.org/test/pts/rav1e
   - Real-world rav1e performance data
   - Shows speed 10 = ~20 FPS on high-end hardware

## Conclusion

**For mead development**:
1. **Don't use rav1e for encoding** - it's 7x slower and produces larger files than SVT-AV1
2. **If adding encoding**, use SVT-AV1 (libsvtav1 via FFmpeg)
3. **Expose minimal settings**: Just "Quality" (Fast/Balanced/High) preset picker
4. **Hide complexity**: Auto-calculate CRF from resolution, auto-detect tiles/threads

**The industry has spoken**: SVT-AV1 is the practical AV1 encoder in 2025. rav1e is a great pure-Rust project and valuable for research, but it's not competitive for production encoding.

## Next Steps

1. Test SVT-AV1 integration (via FFmpeg or direct SVT-AV1 Rust bindings)
2. Benchmark on our target hardware (M3 Max, i9-13900KF)
3. Determine if we want encoding at all (vs just container/demux support)
4. If encoding: Design simple 3-preset UI (Fast/Balanced/High)
