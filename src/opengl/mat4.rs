// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

use euclid::Size2D;

use crate::common::PixelSize;

// 4x4 matrix
pub(crate) struct Mat4([f32; 16]);

impl Mat4 {
    // Orthogonal projection matrix
    #[rustfmt::skip]
    pub(crate) fn projection(size: Size2D<u32, PixelSize>) -> Mat4 {
        assert!(size.width != 0 && size.height != 0);
        let (x, y) = (size.width as f32, size.height as f32);
        Mat4(
            [
                2.0 / x, 0.0     , 0.0, 0.0,
                0.0    , -2.0 / y, 0.0, 0.0,
                0.0    , 0.0     , 0.0, 0.0,
                -1.0   , 1.0     , 0.0, 1.0,
            ]
        )
    }

    /// Get pointer
    pub(crate) fn as_ptr(&self) -> *const f32 {
        self.0.as_ptr()
    }
}

#[cfg(test)]
mod tests {
    use super::Mat4;
    use euclid::size2;

    #[test]
    fn projection() {
        let mat = Mat4::projection(size2(8, 8));
        assert_eq!(
            mat.0,
            [0.25, 0.0, 0.0, 0.0, 0.0, -0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0, 0.0, 1.0]
        );
    }
}
