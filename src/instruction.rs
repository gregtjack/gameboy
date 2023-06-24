use color_eyre::Result;

#[derive(Clone, Copy, Debug)]
pub enum LDPointer {
    /// this is added to 0xFF00 to get the actual address
    Addr8,
    Addr16,
    C,
    BC,
    DE,
    SP,
    HL,
    HLInc,
    HLDec,
}

#[derive(Clone, Copy, Debug)]
pub enum ArithmaticTarget {
    Data,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    /// Pointer to 8-bit value
    HL,
}

#[derive(Clone, Copy, Debug)]
pub enum ArithmaticTarget16 {
    I16,
    BC,
    DE,
    HL,
    SP,
}

#[derive(Clone, Copy, Debug)]
pub enum LDTarget {
    Data,
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    Pointer(LDPointer),
}

#[derive(Clone, Copy, Debug)]
pub enum JumpTarget {
    Addr16,
}

#[derive(Clone, Copy, Debug)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
}

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    // 8-bit load instructions
    LD(LDTarget, LDTarget),

    // 8-bit Arithmatic instructions
    ADD(ArithmaticTarget),
    ADC(ArithmaticTarget),
    SUB(ArithmaticTarget),
    SBC(ArithmaticTarget),
    AND(ArithmaticTarget),
    OR(ArithmaticTarget),
    XOR(ArithmaticTarget),
    CP(ArithmaticTarget),
    INC(ArithmaticTarget),
    DEC(ArithmaticTarget),

    // 16-bit arithmatic instructions
    ADD16(ArithmaticTarget16, ArithmaticTarget16),
    INC16(ArithmaticTarget16),
    DEC16(ArithmaticTarget16),

    // Rotate, shift, and bit instructions
    RLCA,
    RLA,
    RRCA,
    RRA,
    RLC(ArithmaticTarget),
    RL(ArithmaticTarget),
    RRC(ArithmaticTarget),
    RR(ArithmaticTarget),
    SLA(ArithmaticTarget),
    SRA(ArithmaticTarget),
    SRL(ArithmaticTarget),
    SWAP(ArithmaticTarget),
    BIT(ArithmaticTarget),
    SET(ArithmaticTarget),
    RES(ArithmaticTarget),

    // Jumps and calls
    JP(JumpTarget),
    JP_CC(Condition, JumpTarget),
    JR(JumpTarget),
    JR_CC(Condition, JumpTarget),
    RET,
    RET_CC(Condition),
    RETI,
    CALL(Condition, JumpTarget),
    RST,

    // CPU control instructions
    NOP,
    HALT,
    STOP,
    DI,
    EI,
}

pub enum InstructionError {
    NotImplemented,
}

impl Instruction {
    /// decode opcodes into instructions
    pub fn decode(opcode: u8) -> Result<Instruction, InstructionError> {
        match opcode {
            /* CPU control */
            0x00 => Ok(Instruction::NOP),
            0x76 => Ok(Instruction::HALT),
            0x10 => Ok(Instruction::STOP),
            0xF3 => Ok(Instruction::DI),
            0xFB => Ok(Instruction::EI),

            /* 8-bit arithmatic */
            0x80 => Ok(Instruction::ADD(ArithmaticTarget::B)),
            0x81 => Ok(Instruction::ADD(ArithmaticTarget::C)),
            0x82 => Ok(Instruction::ADD(ArithmaticTarget::D)),
            0x83 => Ok(Instruction::ADD(ArithmaticTarget::E)),
            0x84 => Ok(Instruction::ADD(ArithmaticTarget::H)),
            0x85 => Ok(Instruction::ADD(ArithmaticTarget::L)),
            0x86 => Ok(Instruction::ADD(ArithmaticTarget::HL)),
            0x87 => Ok(Instruction::ADD(ArithmaticTarget::A)),
            0xC6 => Ok(Instruction::ADD(ArithmaticTarget::Data)),
            0x88 => Ok(Instruction::ADC(ArithmaticTarget::B)),
            0x89 => Ok(Instruction::ADC(ArithmaticTarget::C)),
            0x8A => Ok(Instruction::ADC(ArithmaticTarget::D)),
            0x8B => Ok(Instruction::ADC(ArithmaticTarget::E)),
            0x8C => Ok(Instruction::ADC(ArithmaticTarget::H)),
            0x8D => Ok(Instruction::ADC(ArithmaticTarget::L)),
            0x8E => Ok(Instruction::ADC(ArithmaticTarget::HL)),
            0x8F => Ok(Instruction::ADC(ArithmaticTarget::A)),
            0xCE => Ok(Instruction::ADC(ArithmaticTarget::Data)),
            0x90 => Ok(Instruction::SUB(ArithmaticTarget::B)),
            0x91 => Ok(Instruction::SUB(ArithmaticTarget::C)),
            0x92 => Ok(Instruction::SUB(ArithmaticTarget::D)),
            0x93 => Ok(Instruction::SUB(ArithmaticTarget::E)),
            0x94 => Ok(Instruction::SUB(ArithmaticTarget::H)),
            0x95 => Ok(Instruction::SUB(ArithmaticTarget::L)),
            0x96 => Ok(Instruction::SUB(ArithmaticTarget::HL)),
            0x97 => Ok(Instruction::SUB(ArithmaticTarget::A)),
            0xD6 => Ok(Instruction::SUB(ArithmaticTarget::Data)),
            0x98 => Ok(Instruction::SBC(ArithmaticTarget::B)),
            0x99 => Ok(Instruction::SBC(ArithmaticTarget::C)),
            0x9A => Ok(Instruction::SBC(ArithmaticTarget::D)),
            0x9B => Ok(Instruction::SBC(ArithmaticTarget::E)),
            0x9C => Ok(Instruction::SBC(ArithmaticTarget::H)),
            0x9D => Ok(Instruction::SBC(ArithmaticTarget::L)),
            0x9E => Ok(Instruction::SBC(ArithmaticTarget::HL)),
            0x9F => Ok(Instruction::SBC(ArithmaticTarget::A)),
            0xDE => Ok(Instruction::SBC(ArithmaticTarget::Data)),
            0xA0 => Ok(Instruction::AND(ArithmaticTarget::B)),
            0xA1 => Ok(Instruction::AND(ArithmaticTarget::C)),
            0xA2 => Ok(Instruction::AND(ArithmaticTarget::D)),
            0xA3 => Ok(Instruction::AND(ArithmaticTarget::E)),
            0xA4 => Ok(Instruction::AND(ArithmaticTarget::H)),
            0xA5 => Ok(Instruction::AND(ArithmaticTarget::L)),
            0xA6 => Ok(Instruction::AND(ArithmaticTarget::HL)),
            0xA7 => Ok(Instruction::AND(ArithmaticTarget::A)),
            0xE6 => Ok(Instruction::AND(ArithmaticTarget::Data)),
            0xA8 => Ok(Instruction::XOR(ArithmaticTarget::B)),
            0xA9 => Ok(Instruction::XOR(ArithmaticTarget::C)),
            0xAA => Ok(Instruction::XOR(ArithmaticTarget::D)),
            0xAB => Ok(Instruction::XOR(ArithmaticTarget::E)),
            0xAC => Ok(Instruction::XOR(ArithmaticTarget::H)),
            0xAD => Ok(Instruction::XOR(ArithmaticTarget::L)),
            0xAE => Ok(Instruction::XOR(ArithmaticTarget::HL)),
            0xAF => Ok(Instruction::XOR(ArithmaticTarget::A)),
            0xEE => Ok(Instruction::XOR(ArithmaticTarget::Data)),
            0xB0 => Ok(Instruction::OR(ArithmaticTarget::B)),
            0xB1 => Ok(Instruction::OR(ArithmaticTarget::C)),
            0xB2 => Ok(Instruction::OR(ArithmaticTarget::D)),
            0xB3 => Ok(Instruction::OR(ArithmaticTarget::E)),
            0xB4 => Ok(Instruction::OR(ArithmaticTarget::H)),
            0xB5 => Ok(Instruction::OR(ArithmaticTarget::L)),
            0xB6 => Ok(Instruction::OR(ArithmaticTarget::HL)),
            0xB7 => Ok(Instruction::OR(ArithmaticTarget::A)),
            0xF6 => Ok(Instruction::OR(ArithmaticTarget::Data)),
            0xB8 => Ok(Instruction::CP(ArithmaticTarget::B)),
            0xB9 => Ok(Instruction::CP(ArithmaticTarget::C)),
            0xBA => Ok(Instruction::CP(ArithmaticTarget::D)),
            0xBB => Ok(Instruction::CP(ArithmaticTarget::E)),
            0xBC => Ok(Instruction::CP(ArithmaticTarget::H)),
            0xBD => Ok(Instruction::CP(ArithmaticTarget::L)),
            0xBE => Ok(Instruction::CP(ArithmaticTarget::HL)),
            0xBF => Ok(Instruction::CP(ArithmaticTarget::A)),
            0xFE => Ok(Instruction::CP(ArithmaticTarget::Data)),
            0x04 => Ok(Instruction::INC(ArithmaticTarget::B)),
            0x0C => Ok(Instruction::INC(ArithmaticTarget::C)),
            0x14 => Ok(Instruction::INC(ArithmaticTarget::D)),
            0x1C => Ok(Instruction::INC(ArithmaticTarget::E)),
            0x24 => Ok(Instruction::INC(ArithmaticTarget::H)),
            0x2C => Ok(Instruction::INC(ArithmaticTarget::L)),
            0x34 => Ok(Instruction::INC(ArithmaticTarget::HL)),
            0x3C => Ok(Instruction::INC(ArithmaticTarget::A)),
            0x05 => Ok(Instruction::DEC(ArithmaticTarget::B)),
            0x0D => Ok(Instruction::DEC(ArithmaticTarget::C)),
            0x15 => Ok(Instruction::DEC(ArithmaticTarget::D)),
            0x1D => Ok(Instruction::DEC(ArithmaticTarget::E)),
            0x25 => Ok(Instruction::DEC(ArithmaticTarget::H)),
            0x2D => Ok(Instruction::DEC(ArithmaticTarget::L)),
            0x35 => Ok(Instruction::DEC(ArithmaticTarget::HL)),
            0x3D => Ok(Instruction::DEC(ArithmaticTarget::A)),

            /* 8-bit load */
            0x40 => Ok(Instruction::LD(LDTarget::B, LDTarget::B)),
            0x41 => Ok(Instruction::LD(LDTarget::B, LDTarget::C)),
            0x42 => Ok(Instruction::LD(LDTarget::B, LDTarget::D)),
            0x43 => Ok(Instruction::LD(LDTarget::B, LDTarget::E)),
            0x44 => Ok(Instruction::LD(LDTarget::B, LDTarget::H)),
            0x45 => Ok(Instruction::LD(LDTarget::B, LDTarget::L)),
            0x46 => Ok(Instruction::LD(
                LDTarget::B,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x47 => Ok(Instruction::LD(LDTarget::B, LDTarget::A)),
            0x48 => Ok(Instruction::LD(LDTarget::C, LDTarget::B)),
            0x49 => Ok(Instruction::LD(LDTarget::C, LDTarget::C)),
            0x4A => Ok(Instruction::LD(LDTarget::C, LDTarget::D)),
            0x4B => Ok(Instruction::LD(LDTarget::C, LDTarget::E)),
            0x4C => Ok(Instruction::LD(LDTarget::C, LDTarget::H)),
            0x4D => Ok(Instruction::LD(LDTarget::C, LDTarget::L)),
            0x4E => Ok(Instruction::LD(
                LDTarget::C,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x4F => Ok(Instruction::LD(LDTarget::C, LDTarget::A)),
            0x50 => Ok(Instruction::LD(LDTarget::D, LDTarget::B)),
            0x51 => Ok(Instruction::LD(LDTarget::D, LDTarget::C)),
            0x52 => Ok(Instruction::LD(LDTarget::D, LDTarget::D)),
            0x53 => Ok(Instruction::LD(LDTarget::D, LDTarget::E)),
            0x54 => Ok(Instruction::LD(LDTarget::D, LDTarget::H)),
            0x55 => Ok(Instruction::LD(LDTarget::D, LDTarget::L)),
            0x56 => Ok(Instruction::LD(
                LDTarget::D,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x57 => Ok(Instruction::LD(LDTarget::D, LDTarget::A)),
            0x58 => Ok(Instruction::LD(LDTarget::E, LDTarget::B)),
            0x59 => Ok(Instruction::LD(LDTarget::E, LDTarget::C)),
            0x5A => Ok(Instruction::LD(LDTarget::E, LDTarget::D)),
            0x5B => Ok(Instruction::LD(LDTarget::E, LDTarget::E)),
            0x5C => Ok(Instruction::LD(LDTarget::E, LDTarget::H)),
            0x5D => Ok(Instruction::LD(LDTarget::E, LDTarget::L)),
            0x5E => Ok(Instruction::LD(
                LDTarget::E,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x5F => Ok(Instruction::LD(LDTarget::E, LDTarget::A)),
            0x60 => Ok(Instruction::LD(LDTarget::H, LDTarget::B)),
            0x61 => Ok(Instruction::LD(LDTarget::H, LDTarget::C)),
            0x62 => Ok(Instruction::LD(LDTarget::H, LDTarget::D)),
            0x63 => Ok(Instruction::LD(LDTarget::H, LDTarget::E)),
            0x64 => Ok(Instruction::LD(LDTarget::H, LDTarget::H)),
            0x65 => Ok(Instruction::LD(LDTarget::H, LDTarget::L)),
            0x66 => Ok(Instruction::LD(
                LDTarget::H,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x67 => Ok(Instruction::LD(LDTarget::H, LDTarget::A)),
            0x68 => Ok(Instruction::LD(LDTarget::L, LDTarget::B)),
            0x69 => Ok(Instruction::LD(LDTarget::L, LDTarget::C)),
            0x6A => Ok(Instruction::LD(LDTarget::L, LDTarget::D)),
            0x6B => Ok(Instruction::LD(LDTarget::L, LDTarget::E)),
            0x6C => Ok(Instruction::LD(LDTarget::L, LDTarget::H)),
            0x6D => Ok(Instruction::LD(LDTarget::L, LDTarget::L)),
            0x6E => Ok(Instruction::LD(
                LDTarget::L,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x6F => Ok(Instruction::LD(LDTarget::L, LDTarget::A)),
            0x70 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::B,
            )),
            0x71 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::C,
            )),
            0x72 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::D,
            )),
            0x73 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::E,
            )),
            0x74 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::H,
            )),
            0x75 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::L,
            )),
            0x77 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::A,
            )),
            0x78 => Ok(Instruction::LD(LDTarget::A, LDTarget::B)),
            0x79 => Ok(Instruction::LD(LDTarget::A, LDTarget::C)),
            0x7A => Ok(Instruction::LD(LDTarget::A, LDTarget::D)),
            0x7B => Ok(Instruction::LD(LDTarget::A, LDTarget::E)),
            0x7C => Ok(Instruction::LD(LDTarget::A, LDTarget::H)),
            0x7D => Ok(Instruction::LD(LDTarget::A, LDTarget::L)),
            0x7E => Ok(Instruction::LD(
                LDTarget::A,
                LDTarget::Pointer(LDPointer::HL),
            )),
            0x7F => Ok(Instruction::LD(LDTarget::A, LDTarget::A)),
            0x02 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::BC),
                LDTarget::A,
            )),
            0x06 => Ok(Instruction::LD(LDTarget::B, LDTarget::Data)),
            0x0E => Ok(Instruction::LD(LDTarget::C, LDTarget::Data)),
            0x12 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::DE),
                LDTarget::A,
            )),
            0x16 => Ok(Instruction::LD(LDTarget::D, LDTarget::Data)),
            0x1A => Ok(Instruction::LD(
                LDTarget::A,
                LDTarget::Pointer(LDPointer::DE),
            )),
            0x1E => Ok(Instruction::LD(LDTarget::E, LDTarget::Data)),
            0x22 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HLInc),
                LDTarget::A,
            )),
            0x26 => Ok(Instruction::LD(LDTarget::H, LDTarget::Data)),
            0x2A => Ok(Instruction::LD(
                LDTarget::A,
                LDTarget::Pointer(LDPointer::HLInc),
            )),
            0x2E => Ok(Instruction::LD(LDTarget::L, LDTarget::Data)),
            0x32 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HLDec),
                LDTarget::A,
            )),
            0x36 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::HL),
                LDTarget::Data,
            )),
            0x3A => Ok(Instruction::LD(
                LDTarget::A,
                LDTarget::Pointer(LDPointer::HLDec),
            )),
            0x3E => Ok(Instruction::LD(LDTarget::A, LDTarget::Data)),

            0xE0 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::Addr8),
                LDTarget::A,
            )),
            0xE2 => Ok(Instruction::LD(
                LDTarget::Pointer(LDPointer::C),
                LDTarget::A,
            )),

            _ => Err(InstructionError::NotImplemented),
        }
    }
}
