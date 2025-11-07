#!/bin/bash
# Compare rav1e (via mead) vs SVT-AV1 encoding performance

set -e

RESOLUTIONS=("1280x720" "1920x1080")
FRAME_COUNT=30
FPS=30

echo "=== AV1 Encoder Comparison: rav1e (mead) vs SVT-AV1 ==="
echo ""

# Check prerequisites
if ! command -v ffmpeg &> /dev/null; then
    echo "ERROR: ffmpeg not found"
    exit 1
fi

if ! command -v SvtAv1EncApp &> /dev/null; then
    echo "ERROR: SvtAv1EncApp not found"
    exit 1
fi

if [ ! -f "./target/release/mead" ]; then
    echo "Building mead in release mode..."
    cargo build --release --package mead
fi

# Cleanup function
cleanup() {
    rm -f /tmp/test_*.y4m /tmp/test_*_mead.ivf /tmp/test_*_svt.ivf
}
trap cleanup EXIT

echo "Test configuration:"
echo "  Frames: $FRAME_COUNT"
echo "  FPS: $FPS"
echo ""

# Results file
RESULTS_FILE="/tmp/encoder_comparison_results.txt"
> $RESULTS_FILE

for RES in "${RESOLUTIONS[@]}"; do
    WIDTH=$(echo $RES | cut -d'x' -f1)
    HEIGHT=$(echo $RES | cut -d'x' -f2)
    DURATION=$(echo "scale=3; $FRAME_COUNT / $FPS" | bc)

    echo "=== Testing $RES ==="

    # Generate test video
    echo "  Generating test Y4M..."
    ffmpeg -f lavfi -i testsrc=duration=$DURATION:size=$RES:rate=$FPS \
        -pix_fmt yuv420p -f yuv4mpegpipe /tmp/test_${RES}.y4m -y 2>/dev/null

    INPUT_SIZE=$(stat -f%z /tmp/test_${RES}.y4m 2>/dev/null || stat -c%s /tmp/test_${RES}.y4m)
    echo "  Input size: $(numfmt --to=iec-i --suffix=B $INPUT_SIZE 2>/dev/null || echo $INPUT_SIZE bytes)"

    # Test mead (rav1e)
    echo "  Encoding with mead (rav1e)..."
    MEAD_START=$(date +%s.%N)
    ./target/release/mead encode /tmp/test_${RES}.y4m \
        -o /tmp/test_${RES}_mead.ivf \
        --codec av1 2>&1 | grep -v "^\[" || true
    MEAD_END=$(date +%s.%N)
    MEAD_TIME=$(echo "$MEAD_END - $MEAD_START" | bc)
    MEAD_FPS=$(echo "scale=2; $FRAME_COUNT / $MEAD_TIME" | bc)
    MEAD_SIZE=$(stat -f%z /tmp/test_${RES}_mead.ivf 2>/dev/null || stat -c%s /tmp/test_${RES}_mead.ivf)

    echo "    Time: ${MEAD_TIME}s"
    echo "    FPS: $MEAD_FPS"
    echo "    Output: $(numfmt --to=iec-i --suffix=B $MEAD_SIZE 2>/dev/null || echo $MEAD_SIZE bytes)"

    # Test SVT-AV1 (speed 6 to match rav1e default)
    echo "  Encoding with SVT-AV1..."
    SVT_START=$(date +%s.%N)
    SvtAv1EncApp -i /tmp/test_${RES}.y4m \
        -b /tmp/test_${RES}_svt.ivf \
        --preset 6 \
        --keyint $FPS \
        --fps $FPS \
        2>&1 | tail -3 || true
    SVT_END=$(date +%s.%N)
    SVT_TIME=$(echo "$SVT_END - $SVT_START" | bc)
    SVT_FPS=$(echo "scale=2; $FRAME_COUNT / $SVT_TIME" | bc)
    SVT_SIZE=$(stat -f%z /tmp/test_${RES}_svt.ivf 2>/dev/null || stat -c%s /tmp/test_${RES}_svt.ivf)

    echo "    Time: ${SVT_TIME}s"
    echo "    FPS: $SVT_FPS"
    echo "    Output: $(numfmt --to=iec-i --suffix=B $SVT_SIZE 2>/dev/null || echo $SVT_SIZE bytes)"

    # Calculate speedup
    SPEEDUP=$(echo "scale=2; $SVT_FPS / $MEAD_FPS" | bc)
    SIZE_RATIO=$(echo "scale=2; $MEAD_SIZE / $SVT_SIZE" | bc)

    echo "  Comparison:"
    echo "    SVT-AV1 is ${SPEEDUP}x faster"
    echo "    Size ratio (mead/svt): ${SIZE_RATIO}x"
    echo ""

    # Store results
    echo "$RES|$MEAD_FPS|$SVT_FPS|$SPEEDUP" >> $RESULTS_FILE
done

echo ""
echo "=== Summary ==="
echo ""
printf "%-12s | %-15s | %-15s | %-10s\n" "Resolution" "mead (rav1e)" "SVT-AV1" "Speedup"
echo "-------------|-----------------|-----------------|------------"

while IFS='|' read -r res mead_fps svt_fps speedup; do
    printf "%-12s | %-15s | %-15s | %-10s\n" "$res" "${mead_fps} fps" "${svt_fps} fps" "${speedup}x"
done < $RESULTS_FILE

echo ""
echo "Notes:"
echo "  - Both encoders used similar quality settings (speed preset 6)"
echo "  - mead uses rav1e with tile parallelism optimization"
echo "  - SVT-AV1 is production encoder used by Netflix, YouTube"
