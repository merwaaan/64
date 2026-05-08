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
    ($base:expr, $($reg_name:ident : $reg_type:ident),* $(,)?) => {
        #[repr(C)]
        #[derive(Default, Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
        pub struct Registers {
            $(pub $reg_name: $reg_type,)*
        }

        mapped_registers!(@reg_impl $base, [], $($reg_type,)*);

        #[derive(PartialEq, Debug, strum::Display, strum::EnumIter)]
        pub enum Register {
            $($reg_type,)*
        }

        impl Register {
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Register::$reg_type => stringify!($reg_name),)*
                }
            }
            pub const fn index(&self) -> usize {
                match self {
                    $(Register::$reg_type => <$reg_type>::INDEX,)*
                }
            }
            pub const fn offset(&self) -> u32 {
                match self {
                    $(Register::$reg_type => <$reg_type>::OFFSET,)*
                }
            }
            pub const fn address(&self) -> u32 {
                match self {
                    $(Register::$reg_type => <$reg_type>::ADDRESS,)*
                }
            }
        }
    };

    (@reg_impl $base:expr, [$($seen:tt)*], $head:ident, $($tail:ident,)*) => {
        impl $head {
            pub const NAME: &'static str = stringify!($reg_name);
            pub const INDEX: usize = mapped_registers!(@reg_count; $($seen)*);
            pub const OFFSET: u32 = (Self::INDEX as u32) * 4;
            pub const ADDRESS: u32 = $base + Self::OFFSET;
        }

        mapped_registers!(@reg_impl $base, [$($seen)* x], $($tail,)*);
    };

    (@reg_impl $base:expr, [$($seen:tt)*],) => {};

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
