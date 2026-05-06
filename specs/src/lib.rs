#![no_std]

// TODO Ai Status masking
// TODO Ai Length masking
// TODO ai DMA enabled
// TODO ai reg mirroring writes?
// TODO vi set/clear behavior

pub mod ai;
pub mod cart;
pub mod color;
pub mod dd;
pub mod interrupt;
pub mod isviewer;
pub mod map;
pub mod mi;
pub mod pi;
pub mod pif;
pub mod rdp;
pub mod si;
pub mod timing;
pub mod vi;

/// TODO doc with test
#[macro_export]
macro_rules! mapped_registers {
    ($base:expr, $($field:ident : $ty:ident),* $(,)?) => {
        #[repr(C)]
        #[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Registers {
            $(pub $field: $ty,)*
        }

        mapped_registers!(@each_impl $base, [], $($ty,)*);
    };

    (@each_impl $base:expr, [$($seen:tt)*], $head:ident, $($tail:ident,)*) => {
        impl $head {
            pub const INDEX: usize = mapped_registers!(@reg_count; $($seen)*);
            pub const OFFSET: u32 = (Self::INDEX as u32) * 4;
            pub const ADDRESS: u32 = $base + Self::OFFSET;
        }

        mapped_registers!(@each_impl $base, [$($seen)* x], $($tail,)*);
    };

    (@each_impl $base:expr, [$($seen:tt)*],) => {};

    (@reg_count ;) => {
        0usize
    };

    (@reg_count ; $($x:tt)+) => {
        0usize $(+ mapped_registers!(@one $x))*
    };

    (@one $x:tt) => {
        1usize
    };
}
