use glam::{DMat3, DMat4, DVec2, DVec3, DVec4, Mat3, Mat4, Vec2, Vec3, Vec4};

/// This trait should be instantiated for all the types transferred to the GPU.
///
/// # Safety
///
/// The vertex attributes need to be correctly communicated to the GPU so no
/// runtime errors occur.
///
/// Simplest way to make the data nicely representable on GPU is using
/// `#[repr(C, packed)]` which makes sure the data is packed.
/// It is also good idea to make sure the structure is 4 byte aligned. For that
/// you can use [u8] padding.
//
// TODO: This is probably a thing that should contain the GPU attributes???
pub unsafe trait Gpu {}

// Yep the more I think about it it should contain the attributes so one can't
// make a mistake.
unsafe impl Gpu for u16 {}
unsafe impl Gpu for u32 {}
unsafe impl Gpu for f32 {}
unsafe impl Gpu for f64 {}

unsafe impl Gpu for Mat3 {}
unsafe impl Gpu for Mat4 {}
unsafe impl Gpu for DMat3 {}
unsafe impl Gpu for DMat4 {}
unsafe impl Gpu for Vec2 {}
unsafe impl Gpu for Vec3 {}
unsafe impl Gpu for Vec4 {}
unsafe impl Gpu for DVec2 {}
unsafe impl Gpu for DVec3 {}
unsafe impl Gpu for DVec4 {}

pub trait Raw {
    fn get_raw(&self) -> &[u8];
    fn byte_len(&self) -> usize;
}

impl<T: Sized + Gpu> Raw for T {
    fn get_raw(&self) -> &[u8] {
        // SAFETY: This slice is supposed to be send to the GPU so the caller
        // must uphold the safety contract of the Gpu trait.
        unsafe {
            core::slice::from_raw_parts(
                (self as *const T) as *const u8,
                std::mem::size_of_val(self),
            )
        }
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl<T: Sized + Gpu> Raw for [T] {
    fn get_raw(&self) -> &[u8] {
        // SAFETY: This slice is supposed to be send to the GPU so the caller
        // must uphold the safety contract of the Gpu trait.
        unsafe {
            core::slice::from_raw_parts(
                (self as *const [T]) as *const u8,
                std::mem::size_of_val(self),
            )
        }
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

impl<T: Sized + Gpu> Raw for Vec<T> {
    fn get_raw(&self) -> &[u8] {
        // SAFETY: This slice is supposed to be send to the GPU so the caller
        // must uphold the safety contract of the Gpu trait.
        unsafe {
            core::slice::from_raw_parts(
                (self.as_slice() as *const [T]) as *const u8,
                std::mem::size_of_val(self.as_slice()),
            )
        }
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of_val(self.as_slice())
    }
}

impl<T: Sized + Gpu, const N: usize> Raw for [T; N] {
    fn get_raw(&self) -> &[u8] {
        // SAFETY: This slice is supposed to be send to the GPU so the caller
        // must uphold the safety contract of the Gpu trait.
        unsafe {
            core::slice::from_raw_parts(
                (self as *const [T]) as *const u8,
                std::mem::size_of_val(self),
            )
        }
    }

    fn byte_len(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
