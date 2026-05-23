use arbitrary_int::prelude::*;
use bitbybit::bitfield;

macro_rules! instructions {
    (
        $(
            $name:ident ($format:ident) = $default:literal {
                $(
                    #[$bits:meta]
                    $field:ident : $field_type:ident
                ),* $(,)?
            }
        ),+ $(,)?
    ) => {
        // Define the instruction structs

        $(
            instructions!(@build $format, $name, $default {
                $( #[$bits] $field: $field_type ),*
            });
        )+

        /// An enum of all the instructions.
        #[derive(Clone, Copy, Debug)]
        pub enum Instruction {
            $( $name($name), )+
        }

        impl Instruction {
            pub fn encode(self) -> u32 {
                match self {
                    $( Self::$name(inner) => inner.raw_value(), )+
                }
            }
        }

        $(
            impl From<$name> for Instruction {
                fn from(inner: $name) -> Self {
                    Self::$name(inner)
                }
            }
        )+
    };

    // TODO rm extra fields?

    // R-type: 000000 [rs (5 bits)] [rt (5 bits)] [rd (5 bits)] TODO shift? TODO opcode

    (@build RType, $name:ident, $default:literal { $( #[$bits:meta] $field:ident : $field_type:ident ),* $(,)? } ) => {
        #[bitfield(u32, forbid_overlaps, introspect, default = $default, debug)]
        pub struct $name {
            #[bits(21..=25, rw)] pub rs: u5,
            #[bits(16..=20, rw)] pub rt: u5,
            #[bits(11..=15, rw)] pub rd: u5,
            $( #[$bits] pub $field: $field_type, )*
        }
    };

    // I-type: [op] [rs (5 bits)] [rt (5 bits)] [imm (16 bits)]

    (@build IType, $name:ident, $default:literal { $( #[$bits:meta] $field:ident : $field_type:ident ),* $(,)? } ) => {
        #[bitfield(u32, forbid_overlaps, introspect, default = $default, debug)]
        pub struct $name {
            #[bits(21..=25, rw)] pub rs: u5,
            #[bits(16..=20, rw)] pub rt: u5,
            #[bits(0..=15, rw)] pub imm: u16,
            $( #[$bits] pub $field: $field_type, )*
        }
    };

    // Custom

    (@build Custom, $name:ident, $default:literal { $( #[$bits:meta] $field:ident : $field_type:ident ),* $(,)? } ) => {
        #[bitfield(u32, forbid_overlaps, introspect, default = $default, debug)]
        pub struct $name {
            $( #[$bits] pub $field: $field_type, )*
        }
    };
}

instructions! {
    // Logical
    And (RType) = 0x0000_0024 {},
    Or (RType) = 0x0000_0025 {},
    Xor (RType) = 0x0000_0026 {},
    Nor (RType) = 0x0000_0027 {},
    Andi (IType) = 0x3000_0000 {},
    Ori (IType) = 0x3400_0000 {},
    Xori (IType) = 0x3800_0000 {},
    //
    Lui (Custom) = 0x3C00_0000 {
        #[bits(16..=20, rw)] rt: u5,
        #[bits(0..=15, rw)] imm: u16,
    },
    Sw (Custom) = 0xAC00_0000 {
        #[bits(21..=25, rw)] base: u5,
        #[bits(16..=20, rw)] rt: u5,
        #[bits(0..=15, rw)] offset: u16,
    },
    //
    Jr (Custom) = 0x0000_0008 {
        #[bits(21..=25, rw)] rs: u5,
    },
}

// pub trait Instruction {
//     fn encode(&self) -> u32;
// }

// #[bitfield(u32, forbid_overlaps, instrospect, default = 0x0000_0024, debug)]
// pub struct And {
//     #[bits(21..=25, rw)]
//     rs: u5,

//     #[bits(16..=20, rw)]
//     rt: u5,

//     #[bits(11..=15, rw)]
//     rd: u5,
// }

// impl Instruction for Ori {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }
// #[bitfield(u32, forbid_overlaps, instrospect, default = 0x0000_0025, debug)]
// pub struct Or {
//     #[bits(21..=25, rw)]
//     rs: u5,

//     #[bits(16..=20, rw)]
//     rt: u5,

//     #[bits(11..=15, rw)]
//     rd: u5,
// }

// impl Instruction for Or {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }

// #[bitfield(u32, forbid_overlaps, instrospect, default = 0x3400_0000, debug)]
// pub struct Ori {
//     #[bits(21..=25, rw)]
//     rs: u5,

//     #[bits(16..=20, rw)]
//     rt: u5,

//     #[bits(0..=15, rw)]
//     immediate: u16,
// }

// impl Instruction for And {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }

// #[bitfield(u32, forbid_overlaps, instrospect, default = 0xAC00_0000, debug)]
// pub struct Sw {
//     #[bits(21..=25, rw)]
//     base: u5,

//     #[bits(16..=20, rw)]
//     rt: u5,

//     #[bits(0..=15, rw)]
//     offset: u16,
// }

// impl Instruction for Sw {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }

// #[bitfield(u32, forbid_overlaps, instrospect, default = 0x3C00_0000, debug)]
// pub struct Lui {
//     #[bits(16..=20, rw)]
//     rt: u5,

//     #[bits(0..=15, rw)]
//     immediate: u16,
// }

// impl Instruction for Lui {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }

// #[bitfield(u32, forbid_overlaps, instrospect, default = 0x0000_0008, debug)]
// pub struct Jr {
//     #[bits(21..=25, rw)]
//     rs: u5,
// }

// impl Instruction for Jr {
//     fn encode(&self) -> u32 {
//         self.raw_value()
//     }
// }
