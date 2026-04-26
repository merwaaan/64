// use crate::{
//     cpu::{
//         decoder::Decoder,
//         instructions::{DisassembleFn, ExecuteFn, Instruction, InstructionResult},
//         opcode::Opcode,
//         operands::Operands,
//     },
//     decode_cop2_x, decode_special_x, decode_standard_x,
//     system::System,
// };

// /// Predecoded instruction with function pointers for execution and disassembly.
// /// Also contains predecoded operands (small operands only: register indices, shift amount).
// #[derive(Clone, Copy)]
// pub struct PredecodedInstruction {
//     pub execute: ExecuteFn,
//     pub disassemble: DisassembleFn,
//     pub operands: Operands,
// }

// // TODO rm?
// // impl Default for PredecodedInstruction {
// //     fn default() -> Self {
// //         Self {
// //             execute: Reserved::execute,
// //             disassemble: Reserved::disassemble,
// //             operands: Operands::default(),
// //         }
// //     }
// // }

// impl PredecodedInstruction {
//     pub const fn for_instruction<I: Instruction>() -> Self {
//         Self {
//             execute: I::execute,
//             disassemble: I::disassemble,
//             operands: Operands::default(),
//         }
//     }
// }

// pub type DecodeFn = fn(Opcode) -> PredecodedInstruction;

// pub struct LutDecoder;

// impl LutDecoder {
//     fn decode(opcode: Opcode) -> PredecodedInstruction {
//         decode_root(opcode)
//     }
// }

// impl Decoder for LutDecoder {
//     fn execute(&self, s: &mut System, op: Opcode) -> InstructionResult {
//         let predecoded = Self::decode(op);
//         (predecoded.execute)(s, op, predecoded.operands)
//     }

//     fn disassemble(&self, s: &System, op: Opcode) -> String {
//         let predecoded = Self::decode(op);
//         (predecoded.disassemble)(s, op, predecoded.operands)
//     }
// }

// pub trait Predecodable: Instruction {
//     fn decode(opcode: Opcode) -> PredecodedInstruction;
// }

// // Generate lookup tables for the different instruction groups
// //
// // The predecode_xxx macros generate decode() implementations for a set of instructions.
// // Each instruction has a static LUT of PredecodedInstruction instances, one for each possible operand combination.

// macro_rules! predecode_empty {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(_opcode: Opcode) -> PredecodedInstruction {
//                 PredecodedInstruction::for_instruction::<$inst>()
//             }
//         })*
//     };
// }

// predecode_empty!(
//     crate::cpu::instructions::Reserved,
//     crate::cpu::instructions::special::Break,
//     crate::cpu::instructions::special::Sync,
//     crate::cpu::instructions::special::Syscall,
//     crate::cpu::instructions::standard::Cache,
//     crate::cpu::instructions::standard::J,
//     crate::cpu::instructions::standard::Jal,
//     crate::cpu::instructions::standard::Lui,
// );

// macro_rules! predecode_rs {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32] = {
//                     let mut lut = [PredecodedInstruction {
//                         execute: <$inst>::execute,
//                         disassemble: <$inst>::disassemble,
//                         operands: Operands::default(),
//                     }; 32];

//                     let mut rs = 0;
//                     while rs < 32 {
//                         lut[rs].operands.rs = rs as u8;
//                         rs += 1;
//                     }

//                     lut
//                 };

//                 LUT[(opcode.rs() >> 21) & 0x1F]
//             }
//         })*
//     };
// }

// predecode_rs!(
//     crate::cpu::instructions::regimm::Bgez,
//     crate::cpu::instructions::regimm::Bgezal,
//     crate::cpu::instructions::regimm::Bltz,
//     crate::cpu::instructions::regimm::Bltzal,
//     crate::cpu::instructions::standard::Blez,
//     crate::cpu::instructions::standard::Blezl,
//     crate::cpu::instructions::standard::Bgtz,
//     crate::cpu::instructions::standard::Bgtzl,
//     crate::cpu::instructions::special::Jr,
//     crate::cpu::instructions::special::Mthi,
//     crate::cpu::instructions::special::Mtlo,
// );

// macro_rules! predecode_rd {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32];

//                     let mut rd = 0;
//                     while rd < 32 {
//                         lut[rd].operands.rd = rd as u8;
//                         rd += 1;
//                     }

//                     lut
//                 };

//                 LUT[(opcode.rd() >> 11) & 0x1F]
//             }
//         })*
//     };
// }

// predecode_rd!(
//     crate::cpu::instructions::special::Mfhi,
//     crate::cpu::instructions::special::Mflo,
// );

// macro_rules! predecode_rs_rt {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction ; 32 * 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32 * 32];

//                     let mut rs = 0;
//                     while rs < 32 {
//                         let mut rt = 0;
//                         while rt < 32 {
//                             let key = (rs << 5) | rt;
//                             lut[key].operands.rt = rt as u8;
//                             lut[key].operands.rs = rs as u8;
//                             rt += 1;
//                         }
//                         rs += 1;
//                     }

//                     lut
//                 };

//                 LUT[((opcode.0 >> 16) & 0x3FF) as usize]
//             }
//         })*
//     };
// }

// predecode_rs_rt!(
//     crate::cpu::instructions::standard::Addi,
//     crate::cpu::instructions::standard::Addiu,
//     crate::cpu::instructions::standard::Andi,
//     crate::cpu::instructions::standard::Beq,
//     crate::cpu::instructions::standard::Beql,
//     crate::cpu::instructions::standard::Bne,
//     crate::cpu::instructions::standard::Bnel,
//     crate::cpu::instructions::standard::Daddi,
//     crate::cpu::instructions::standard::Daddiu,
//     crate::cpu::instructions::standard::Lb,
//     crate::cpu::instructions::standard::Lbu,
//     crate::cpu::instructions::standard::Ldl,
//     crate::cpu::instructions::standard::Ldr,
//     crate::cpu::instructions::standard::Ld,
//     crate::cpu::instructions::standard::Ldc1,
//     crate::cpu::instructions::standard::Lh,
//     crate::cpu::instructions::standard::Lhu,
//     crate::cpu::instructions::standard::Ll,
//     crate::cpu::instructions::standard::Lld,
//     crate::cpu::instructions::standard::Lw,
//     crate::cpu::instructions::standard::Lwc1,
//     crate::cpu::instructions::standard::Lwl,
//     crate::cpu::instructions::standard::Lwr,
//     crate::cpu::instructions::standard::Lwu,
//     crate::cpu::instructions::standard::Sb,
//     crate::cpu::instructions::standard::Sc,
//     crate::cpu::instructions::standard::Scd,
//     crate::cpu::instructions::standard::Sd,
//     crate::cpu::instructions::standard::Sdc1,
//     crate::cpu::instructions::standard::Sdl,
//     crate::cpu::instructions::standard::Sdr,
//     crate::cpu::instructions::standard::Sh,
//     crate::cpu::instructions::standard::Slti,
//     crate::cpu::instructions::standard::Sltiu,
//     crate::cpu::instructions::standard::Sw,
//     crate::cpu::instructions::standard::Swc1,
//     crate::cpu::instructions::standard::Swl,
//     crate::cpu::instructions::standard::Swr,
//     crate::cpu::instructions::standard::Ori,
//     crate::cpu::instructions::standard::Xori,
//     crate::cpu::instructions::special::Ddiv,
//     crate::cpu::instructions::special::Ddivu,
//     crate::cpu::instructions::special::Div,
//     crate::cpu::instructions::special::Divu,
//     crate::cpu::instructions::special::Dmult,
//     crate::cpu::instructions::special::Dmultu,
//     crate::cpu::instructions::special::Mult,
//     crate::cpu::instructions::special::Multu,
//     crate::cpu::instructions::special::Sub,
//     crate::cpu::instructions::special::Subu,
//     crate::cpu::instructions::special::Tge,
//     crate::cpu::instructions::special::Tgeu,
//     crate::cpu::instructions::special::Tlt,
//     crate::cpu::instructions::special::Tltu,
//     crate::cpu::instructions::special::Teq,
//     crate::cpu::instructions::special::Tne,
// );

// macro_rules! predecode_rt_rd {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32 * 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32 * 32];

//                     let mut rt = 0;
//                     while rt < 32 {
//                         let mut rd = 0;
//                         while rd < 32 {
//                             let key = (rt << 5) | rd;
//                             lut[key].operands.rt = rt as u8;
//                             lut[key].operands.rd = rd as u8;
//                             rd += 1;
//                         }
//                         rt += 1;
//                     }

//                     lut
//                 };

//                 LUT[((opcode.0 >> 11) & 0x3FF) as usize]
//             }
//         })*
//     };
// }

// predecode_rt_rd!(
//     crate::cpu::instructions::cop2::Mfc2,
//     crate::cpu::instructions::cop2::Dmfc2,
//     crate::cpu::instructions::cop2::Cfc2,
//     crate::cpu::instructions::cop2::Mtc2,
//     crate::cpu::instructions::cop2::Dmtc2,
//     crate::cpu::instructions::cop2::Ctc2,
// );

// macro_rules! predecode_rs_rd {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32 * 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32 * 32];

//                     let mut rs = 0u32;
//                     while rs < 32 {
//                         let mut rd = 0u32;
//                         while rd < 32 {
//                             let key = (rs << 5 | rd) as usize;
//                             lut[key].operands.rs = rs as u8;
//                             lut[key].operands.rd = rd as u8;
//                             rd += 1;
//                         }
//                         rs += 1;
//                     }

//                     lut
//                 };

//                 LUT[(opcode.rs() << 5 | opcode.rd()) as usize]
//             }
//         })*
//     };
// }

// predecode_rs_rd!(crate::cpu::instructions::special::Jalr);

// macro_rules! predecode_rd_rt_rs {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32 * 32 * 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32 * 32 * 32];

//                     let mut rd = 0u32;
//                     while rd < 32 {
//                         let mut rt = 0u32;
//                         while rt < 32 {
//                             let mut rs = 0u32;
//                             while rs < 32 {
//                                 let key = (rd << 10 | rt << 5 | rs) as usize;
//                                 lut[key].operands.rd = rd as u8;
//                                 lut[key].operands.rt = rt as u8;
//                                 lut[key].operands.rs = rs as u8;
//                                 rs += 1;
//                             }
//                             rt += 1;
//                         }
//                         rd += 1;
//                     }

//                     lut
//                 };

//                 LUT[((opcode.0 >> 11) & 0x7FFF) as usize]
//             }
//         })*
//     };
// }

// predecode_rd_rt_rs!(
//     crate::cpu::instructions::special::Add,
//     crate::cpu::instructions::special::Addu,
//     crate::cpu::instructions::special::And,
//     crate::cpu::instructions::special::Dadd,
//     crate::cpu::instructions::special::Daddu,
//     crate::cpu::instructions::special::Dsllv,
//     crate::cpu::instructions::special::Dsrav,
//     crate::cpu::instructions::special::Dsrlv,
//     crate::cpu::instructions::special::Dsub,
//     crate::cpu::instructions::special::Dsubu,
//     crate::cpu::instructions::special::Nor,
//     crate::cpu::instructions::special::Or,
//     crate::cpu::instructions::special::Sllv,
//     crate::cpu::instructions::special::Slt,
//     crate::cpu::instructions::special::Sltu,
//     crate::cpu::instructions::special::Srav,
//     crate::cpu::instructions::special::Srlv,
//     crate::cpu::instructions::special::Xor,
// );

// macro_rules! predecode_rd_rt_sa {
//     ($($inst:path),+ $(,)?) => {$(
//         impl Predecodable for $inst {
//             fn decode(opcode: Opcode) -> PredecodedInstruction {
//                 static LUT: [PredecodedInstruction; 32 * 32 * 32] = {
//                     let mut lut = [PredecodedInstruction::for_instruction::<$inst>(); 32 * 32 * 32];

//                     let mut rd = 0u32;
//                     while rd < 32 {
//                         let mut rt = 0u32;
//                         while rt < 32 {
//                             let mut sa = 0u32;
//                             while sa < 32 {
//                                 let key = (rd << 10 | rt << 5 | sa) as usize;
//                                 lut[key].operands.rd = rd as u8;
//                                 lut[key].operands.rt = rt as u8;
//                                 lut[key].operands.sa = sa as u8;
//                                 sa += 1;
//                             }
//                             rt += 1;
//                         }
//                         rd += 1;
//                     }

//                     lut
//                 };

//                 LUT[((opcode.0 >> 6) & 0x7FFF) as usize]
//             }
//         })*
//     };
// }

// predecode_rd_rt_sa!(
//     crate::cpu::instructions::special::Dsll,
//     crate::cpu::instructions::special::Dsll32,
//     crate::cpu::instructions::special::Dsra,
//     crate::cpu::instructions::special::Dsra32,
//     crate::cpu::instructions::special::Dsrl,
//     crate::cpu::instructions::special::Dsrl32,
//     crate::cpu::instructions::special::Sll,
//     crate::cpu::instructions::special::Sra,
//     crate::cpu::instructions::special::Srl,
// );

// // // TODO temp
// // fn fake_decode_fn(_opcode: Opcode) -> PredecodedInstruction {
// //     panic!("fake_decode_fn");
// // }

// // fn fake_execute(_s: &mut System, _opcode: Opcode, _operands: Operands) -> InstructionResult {
// //     panic!("fake_execute");
// // }

// // fn fake_disassemble(_s: &System, _opcode: Opcode, _operands: Operands) -> String {
// //     panic!("fake_disassemble");
// // }

// fn decode_reserved(_opcode: Opcode) -> PredecodedInstruction {
//     crate::cpu::instructions::Reserved::decode(_opcode)
// }

// /// Root decoder for top-level instructions or sub-groups (handled by their own decoders).
// fn decode_all(opcode: Opcode) -> PredecodedInstruction {
//     static LUT: [DecodeFn; 64] = {
//         let mut lut: [DecodeFn; 64] = [decode_reserved; 64];

//         let mut group = 0;
//         while group < 64 {
//             let opcode = Opcode((group << 26) as u32);

//             macro_rules! add_to_lut {
//                 ($inst:path) => {
//                     lut[group] = <$inst>::decode
//                 };
//             }

//             decode_standard_x!(opcode, add_to_lut);

//             group += 1;
//         }

//         lut[0b000000] = decode_special;
//         lut[0b000001] = decode_regimm;
//         lut[0b010000] = decode_cop0;
//         // lut[0b010001] = decode_cop1; // TODO
//         lut[0b010010] = decode_cop2;

//         lut
//     };

//     LUT[opcode.group() as usize](opcode)
// }

// /// Root decoder for top-level instructions or sub-groups (handled by their own decoders).
// fn decode_root(opcode: Opcode) -> PredecodedInstruction {
//     static LUT: [DecodeFn; 64] = {
//         let mut lut: [DecodeFn; 64] = [decode_reserved; 64];

//         let mut group = 0;
//         while group < 64 {
//             let opcode = Opcode((group << 26) as u32);

//             macro_rules! add_to_lut {
//                 ($inst:path) => {
//                     lut[group] = <$inst>::decode
//                 };
//             }

//             decode_standard_x!(opcode, add_to_lut);

//             group += 1;
//         }

//         lut[0b000000] = decode_special;
//         lut[0b000001] = decode_regimm;
//         lut[0b010000] = decode_cop0;
//         // lut[0b010001] = decode_cop1; // TODO
//         lut[0b010010] = decode_cop2;

//         lut
//     };

//     LUT[opcode.group() as usize](opcode)
// }

// fn decode_special(opcode: Opcode) -> PredecodedInstruction {
//     static LUT: [DecodeFn; 64] = {
//         let mut lut: [DecodeFn; 64] = [decode_reserved; 64];

//         let mut sub = 0usize;
//         while sub < 64 {
//             let opcode = Opcode(sub as u32);

//             macro_rules! add_to_lut {
//                 ($inst:path) => {
//                     lut[sub] = <$inst>::decode
//                 };
//             }

//             decode_special_x!(opcode, add_to_lut);

//             sub += 1;
//         }

//         lut
//     };

//     LUT[(opcode.0 & 0x3F) as usize](opcode)
// }

// fn decode_regimm(opcode: Opcode) -> PredecodedInstruction {
//     static LUT: [DecodeFn; 64] = {
//         let mut lut: [DecodeFn; 64] = [decode_reserved; 64];

//         let mut sub = 0usize;
//         while sub < 64 {
//             let opcode = Opcode((sub << 16) as u32);

//             macro_rules! add_to_lut {
//                 ($inst:path) => {
//                     lut[sub] = <$inst>::decode
//                 };
//             }

//             decode_special_x!(opcode, add_to_lut);

//             sub += 1;
//         }

//         lut
//     };

//     LUT[((opcode.0 >> 16) & 0x1F) as usize](opcode)
// }

// fn decode_cop2(opcode: Opcode) -> PredecodedInstruction {
//     static LUT: [DecodeFn; 32] = {
//         let mut lut: [DecodeFn; 32] = [decode_reserved; 32];

//         let mut sub = 0usize;
//         while sub < 32 {
//             let opcode = Opcode((sub << 21) as u32);

//             macro_rules! add_to_lut {
//                 ($inst:path) => {
//                     lut[sub] = <$inst>::decode
//                 };
//             }

//             decode_cop2_x!(opcode, add_to_lut);

//             sub += 1;
//         }

//         lut
//     };

//     LUT[((opcode.0 >> 21) & 0x1F) as usize](opcode)
// }
