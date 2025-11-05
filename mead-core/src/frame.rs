//! Video frame data structures

use aligned_vec::AVec;
use std::sync::Arc;

/// Pixel format for video frames
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// YUV 4:2:0 planar (most common)
    Yuv420p,
    /// YUV 4:2:2 planar
    Yuv422p,
    /// YUV 4:4:4 planar
    Yuv444p,
    /// RGB 24-bit
    Rgb24,
}

/// A single plane of pixel data
///
/// Uses SIMD-aligned memory (AVec) for optimal performance
/// with vectorized operations (SSE, AVX, NEON).
#[derive(Debug, Clone)]
pub struct Plane {
    /// Aligned pixel data
    data: AVec<u8>,
    /// Stride (bytes per row) - may be larger than width due to alignment
    stride: usize,
    /// Plane width in pixels
    width: usize,
    /// Plane height in pixels
    height: usize,
}

impl Plane {
    /// Create a new plane with specified dimensions
    ///
    /// Data is allocated with SIMD alignment (32 bytes)
    pub fn new(width: usize, height: usize) -> Self {
        let stride = width; // For now, stride = width
        let size = stride * height;
        let data = AVec::from_iter(32, std::iter::repeat_n(0, size));

        Self {
            data,
            stride,
            width,
            height,
        }
    }

    /// Create a plane from existing data
    pub fn from_data(data: Vec<u8>, width: usize, height: usize, stride: usize) -> Self {
        let aligned_data = AVec::from_iter(32, data);
        Self {
            data: aligned_data,
            stride,
            width,
            height,
        }
    }

    /// Get the raw data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable raw data
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Get stride (bytes per row)
    pub fn stride(&self) -> usize {
        self.stride
    }

    /// Get width in pixels
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get height in pixels
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get a row of pixels
    pub fn row(&self, y: usize) -> &[u8] {
        let start = y * self.stride;
        let end = start + self.width;
        &self.data[start..end]
    }

    /// Get a mutable row of pixels
    pub fn row_mut(&mut self, y: usize) -> &mut [u8] {
        let start = y * self.stride;
        let end = start + self.width;
        &mut self.data[start..end]
    }
}

/// A decoded video frame with multiple planes
///
/// Typically used with Arc for zero-copy sharing between pipeline stages.
/// For YUV formats, planes are ordered as [Y, U, V].
#[derive(Debug, Clone)]
pub struct Frame {
    /// Pixel planes (Y, U, V for YUV formats)
    planes: Vec<Plane>,
    /// Frame width (full resolution, not subsampled)
    width: u32,
    /// Frame height (full resolution, not subsampled)
    height: u32,
    /// Pixel format
    format: PixelFormat,
    /// Presentation timestamp
    pts: Option<i64>,
}

impl Frame {
    /// Create a new frame with specified format and dimensions
    pub fn new(width: u32, height: u32, format: PixelFormat) -> Self {
        let planes = match format {
            PixelFormat::Yuv420p => {
                // Y plane: full resolution
                // U, V planes: half resolution (4:2:0 subsampling)
                vec![
                    Plane::new(width as usize, height as usize),
                    Plane::new((width / 2) as usize, (height / 2) as usize),
                    Plane::new((width / 2) as usize, (height / 2) as usize),
                ]
            }
            PixelFormat::Yuv422p => {
                // Y plane: full resolution
                // U, V planes: half width (4:2:2 subsampling)
                vec![
                    Plane::new(width as usize, height as usize),
                    Plane::new((width / 2) as usize, height as usize),
                    Plane::new((width / 2) as usize, height as usize),
                ]
            }
            PixelFormat::Yuv444p => {
                // All planes full resolution
                vec![
                    Plane::new(width as usize, height as usize),
                    Plane::new(width as usize, height as usize),
                    Plane::new(width as usize, height as usize),
                ]
            }
            PixelFormat::Rgb24 => {
                // Single interleaved RGB plane
                vec![Plane::new((width * 3) as usize, height as usize)]
            }
        };

        Self {
            planes,
            width,
            height,
            format,
            pts: None,
        }
    }

    /// Get frame width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get frame height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get pixel format
    pub fn format(&self) -> PixelFormat {
        self.format
    }

    /// Get presentation timestamp
    pub fn pts(&self) -> Option<i64> {
        self.pts
    }

    /// Set presentation timestamp
    pub fn set_pts(&mut self, pts: i64) {
        self.pts = Some(pts);
    }

    /// Get reference to planes
    pub fn planes(&self) -> &[Plane] {
        &self.planes
    }

    /// Get mutable reference to planes
    pub fn planes_mut(&mut self) -> &mut [Plane] {
        &mut self.planes
    }

    /// Get Y plane (luma) for YUV formats
    pub fn plane_y(&self) -> Option<&Plane> {
        matches!(self.format, PixelFormat::Yuv420p | PixelFormat::Yuv422p | PixelFormat::Yuv444p)
            .then(|| &self.planes[0])
    }

    /// Get U plane (chroma) for YUV formats
    pub fn plane_u(&self) -> Option<&Plane> {
        matches!(self.format, PixelFormat::Yuv420p | PixelFormat::Yuv422p | PixelFormat::Yuv444p)
            .then(|| &self.planes[1])
    }

    /// Get V plane (chroma) for YUV formats
    pub fn plane_v(&self) -> Option<&Plane> {
        matches!(self.format, PixelFormat::Yuv420p | PixelFormat::Yuv422p | PixelFormat::Yuv444p)
            .then(|| &self.planes[2])
    }
}

/// Type alias for reference-counted frames (zero-copy sharing)
pub type ArcFrame = Arc<Frame>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plane_creation() {
        let plane = Plane::new(64, 64);
        assert_eq!(plane.width(), 64);
        assert_eq!(plane.height(), 64);
        assert_eq!(plane.data().len(), 64 * 64);
    }

    #[test]
    fn test_plane_row_access() {
        let mut plane = Plane::new(4, 4);
        plane.row_mut(0).copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(plane.row(0), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_yuv420p_frame() {
        let frame = Frame::new(64, 64, PixelFormat::Yuv420p);
        assert_eq!(frame.width(), 64);
        assert_eq!(frame.height(), 64);
        assert_eq!(frame.planes().len(), 3);

        // Y plane: full resolution
        assert_eq!(frame.planes()[0].width(), 64);
        assert_eq!(frame.planes()[0].height(), 64);

        // U, V planes: half resolution (4:2:0)
        assert_eq!(frame.planes()[1].width(), 32);
        assert_eq!(frame.planes()[1].height(), 32);
        assert_eq!(frame.planes()[2].width(), 32);
        assert_eq!(frame.planes()[2].height(), 32);
    }

    #[test]
    fn test_yuv422p_frame() {
        let frame = Frame::new(64, 64, PixelFormat::Yuv422p);
        assert_eq!(frame.planes().len(), 3);

        // Y plane: full resolution
        assert_eq!(frame.planes()[0].width(), 64);
        assert_eq!(frame.planes()[0].height(), 64);

        // U, V planes: half width (4:2:2)
        assert_eq!(frame.planes()[1].width(), 32);
        assert_eq!(frame.planes()[1].height(), 64);
    }

    #[test]
    fn test_frame_arc() {
        let frame = Arc::new(Frame::new(64, 64, PixelFormat::Yuv420p));
        let _frame2 = frame.clone(); // Only increments refcount
        assert_eq!(Arc::strong_count(&frame), 2);
    }

    #[test]
    fn test_plane_data_is_aligned() {
        let plane = Plane::new(64, 64);
        let ptr = plane.data().as_ptr() as usize;
        assert_eq!(ptr % 32, 0); // 32-byte alignment for AVX
    }
}
