use arbitrary_int::prelude::*;
use bitbybit::bitfield;

macro_rules! instructions {
    (
        $(
            $name:ident = $default:literal { $($fields:ident)* }
        ),+ $(,)?
    ) => {
        // One struct for each instruction type

        $(
            instructions! { @build_struct $name $default $( $fields )* }
        )+

        /// An enum of all the instructions.
        #[derive(Clone, Copy, Debug)]
        pub enum Instruction {
            $( $name($name), )+
        }

        impl Instruction {
            /// Returns the raw opcode of the instruction.
            pub fn opcode(self) -> u32 { // TODO rename opcode
                match self {
                    $( Self::$name(inner) => inner.raw_value(), )+
                }
            }
        }

        // Instruction types to Instruction enum conversions

        $(
            impl From<$name> for Instruction {
                fn from(inner: $name) -> Self {
                    Self::$name(inner)
                }
            }
        )+
    };

    // Muncher for instruction fields
    //
    // Consumes fields (rt, rs, etc) until the list is empty and them emits the final struct

    (@build_struct $name:ident $default:literal $( $field:ident )*) => {
        instructions! { @build_struct [$name, $default] [] $( $field )* }
    };

    // rs
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] rs $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(21..=25, rw)] pub rs: u5,] $($rest)*
        }
    };

    // rt
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] rt $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(16..=20, rw)] pub rt: u5,] $($rest)*
        }
    };

    // rd
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] rd $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(11..=15, rw)] pub rd: u5,] $($rest)*
        }
    };

    // sa
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] sa $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(6..=10, rw)] pub sa: u5,] $($rest)*
        }
    };

    // imm
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] imm $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(0..=15, rw)] pub imm: u16,] $($rest)*
        }
    };

    // base
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] base $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(21..=25, rw)] pub base: u5,] $($rest)*
        }
    };

    // offset
    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] offset $($rest:tt)*) => {
        instructions! { @build_struct [$name, $default]
            [$($body)* #[bits(0..=15, rw)] pub offset: u16,] $($rest)*
        }
    };

    (@build_struct [$name:ident, $default:literal] [$($body:tt)*] $unknown:ident $($rest:tt)*) => {
        compile_error!(concat!("unsupported field: ", stringify!($unknown)));
    };

    (@build_struct [$name:ident, $default:literal] [$($body:tt)*]) => {
        #[bitfield(u32, forbid_overlaps, introspect, default = $default, debug)]
        pub struct $name {
            $($body)*
        }
    };
}

instructions! {
    // Arithmetic
    Mult = 0x0000_0018 { rs rt },
    Multu = 0x0000_0019 { rs rt },
    Add = 0x0000_0020 { rs rt rd },
    Addu = 0x0000_0021 { rs rt rd },
    Sub = 0x0000_0022 { rs rt rd },
    Subu = 0x0000_0023 { rs rt rd },
    Dadd = 0x0000_002C { rs rt rd },
    Dsub = 0x0000_002E { rs rt rd },
    Dsubu = 0x0000_002F { rs rt rd },
    Addi = 0x2000_0000 { rs rt imm },
    Addiu = 0x2400_0000 { rs rt imm },
    Daddi = 0x6000_0000 { rs rt imm },
    Daddiu = 0x6400_0000 { rs rt imm },
    Mfhi = 0x0000_0010 { rd },
    Mthi = 0x0000_0011 { rs },
    Mflo = 0x0000_0012 { rd },
    Mtlo = 0x0000_0013 { rs },
    // Logical
    And = 0x0000_0024 { rs rt rd },
    Or = 0x0000_0025 { rs rt rd },
    Xor = 0x0000_0026 { rs rt rd },
    Nor = 0x0000_0027 { rs rt rd },
    Andi = 0x3000_0000 { rs rt imm },
    Ori = 0x3400_0000 { rs rt imm },
    Xori = 0x3800_0000 { rs rt imm },
    // Shifts
    Sll = 0x0000_0000 { rt rd sa },
    Srl = 0x0000_0002 { rt rd sa },
    Sra = 0x0000_0003 { rt rd sa },
    Sllv = 0x0000_0004 { rs rt rd },
    Srlv = 0x0000_0006 { rs rt rd },
    Srav = 0x0000_0007 { rs rt rd },
    Dsllv = 0x0000_0014 { rs rt rd },
    Dsrlv = 0x0000_0016 { rs rt rd },
    Dsrav = 0x0000_0017 { rs rt rd },
    Dsll = 0x0000_0038 { rt rd sa },
    Dsrl = 0x0000_003A { rt rd sa },
    Dsra = 0x0000_003B { rt rd sa },
    Dsll32 = 0x0000_003C { rt rd sa },
    Dsrl32 = 0x0000_003E { rt rd sa },
    Dsra32 = 0x0000_003F { rt rd sa },
    // Loads/Stores
    Lui = 0x3C00_0000 { rt imm },
    Lb = 0x8000_0000 { base rt offset },
    Lh = 0x8400_0000 { base rt offset },
    Lbu = 0x9000_0000 { base rt offset },
    Lwu = 0x9C00_0000 { base rt offset },
    Sb = 0xA000_0000 { base rt offset },
    Sw = 0xAC00_0000 { base rt offset },

    Ld = 0xDC00_0000 { base rt offset },
    Sd = 0xFC00_0000 { base rt offset },
    // Jumps
    Jr = 0x0000_0008 { rs },
    // Coprocessor 0
    Mfc0 = 0x4000_0000 { rt rd },
    Mtc0 = 0x4080_0000 { rt rd },
    Dmtc0 = 0x40A0_0000 { rt rd },
    Dmfc0 = 0x4020_0000 { rt rd },
    Tlbr = 0x4200_0001 { },
    Tlbwi = 0x4200_0002 { },
    Tlbwr = 0x4200_0006 { },
    Tlbp = 0x1000_0008 { },
    Eret = 0x4200_0018 { },
    // TODOCache = 0x1000_001C { },
}
