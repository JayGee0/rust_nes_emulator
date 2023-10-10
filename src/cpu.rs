use crate::opcodes::OPCODES_MAP;
use bitflags::bitflags;

bitflags! {
    pub struct CPUFlags: u8 {
        const NEGATIVE   = 0b1000_0000;
        const OVERFLOW   = 0b0100_0000;
        const BREAK1     = 0b0010_0000;
        const BREAK2     = 0b0001_0000;
        const DECIMAL    = 0b0000_1000;
        const INTERRUPT  = 0b0000_0100;
        const ZERO       = 0b0000_0010;
        const CARRY      = 0b0000_0001;
        
    }
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    
    pub register_s: u8,
    /*
    NVss DIZC
    7_______0

    N - Negative
    V - Overflow
    ss - B flag, doesn't affect CPU operation, 1 by default
    
    D - Decimal
    I - Interrupt Disable Signal
    Z - Zero 
    C - Carry
    
     */
    pub status: CPUFlags,
    pub program_counter: u16,
    memory: [u8; 0xFFFF] // Array of u8 of length 0xFFFF
    
    
}

#[derive(Debug)]
#[allow(non_camel_case_types)]

pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    
    Absolute,
    Absolute_X,
    Absolute_Y,
    
    Indirect_X,
    Indirect_Y,
    
    NoneAddressing
}

impl CPU {
    pub fn new() -> Self {

        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            register_s: 0xFF,
            status: CPUFlags::empty(),
            
            program_counter: 0,
            memory: [0; 0xFFFF],
            
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
        
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
        
    }

    fn mem_read_u16(&mut self, pos: u16) -> u16 {
        let lo = self.mem_read(pos) as u16; // Reading the data from pos
        let hi = self.mem_read(pos + 1) as u16; // Reading the next data
        // Shifting the next data left by 8 bits and replacing 
        
        // The empty 0s with the bits from the first 8 bits
        
        (hi << 8) | (lo as u16) 
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8; 
        let lo = (data & 0xFF) as u8;
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi)
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        
        self.register_s = 0xFF;
        self.status = CPUFlags::empty();
        
        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        
        
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    // Where am I addressing data from?
    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {

        match mode {
             // Immediate addressing deals with the number itself
             // e.g. LDA #10 means load 10 into the accumulator
            AddressingMode::Immediate => self.program_counter,

            // Address data from the next 8 bits only (the first 256 bytes of memory)
            
            
            
            // Good for conserving speed and memory
            
            // e.g. LDA $00 loads accumulator from $0000
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            // Address data from the absolute address
            // e.g. LDA $1234 load from $1234 into accumulator
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            // Same as ZeroPage, however with the inclusion of adding the
            // value from register_x to the address
            // e.g. LDA $00,X
            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            },

            // Same as ZeroPage_X except with Y this time. Can only be used with LDX and SDX
            
            
            // e.g. LDX $10,Y
            
            AddressingMode::ZeroPage_Y => {
                
                let pos = self.mem_read(self.program_counter);
                let addr = pos.wrapping_add(self.register_y) as u16;
                
                addr
            },

            // Same as Absolute, however with adding the value from register_x
            // e.g. LDA $1000,X
            AddressingMode::Absolute_X => {
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_x as u16);
                addr
            },

            // Same as Absolute, however with adding the value from register_y
            
            // e.g. LDA $1000,Y
            
            AddressingMode::Absolute_Y => {
                
                let pos = self.mem_read_u16(self.program_counter);
                let addr = pos.wrapping_add(self.register_y as u16);
                
                addr
            },
            
            // Indexed Indirect, Address taken from table of addresses held on the zero page.
            // Tabled address taken from instruction and value of register_x added to give the location of
            // the LSB of the target
            // e.g. LDA ($00, X)
            AddressingMode::Indirect_X => {
                let base = self.mem_read(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            // Indirect Indexed, Zero page location of the LSB of 16 bit address + register_y
            
            AddressingMode::Indirect_Y => {
                
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                
                deref
            }

            AddressingMode::NoneAddressing => {
                panic!("Mode {:?} is not supported", mode)
            }

        }

    }

    // https://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    // Add with Carry
    fn adc(&mut self, mode: &AddressingMode) {
        let addr: u16 = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.add_to_acc_with_carry(data);
    }   

    // Logical AND
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a &= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl_acc(&mut self) {
        let mut data = self.register_a;
        data = self.shift_left(data);
        self.register_a = data;
    }

    // Arithmetic Shift Left 
    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);

        data = self.shift_left(data);
        self.mem_write(addr, data);
    }

    fn shift_left(&mut self, mut data: u8) -> u8 {
        self.status.set(CPUFlags::CARRY, data & 0b1000_0000 > 0);
        data = data << 1;
        self.update_zero_and_negative_flags(data);
        return data;
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.status.set(CPUFlags::ZERO, self.register_a & data == 0);
        self.status.set(CPUFlags::OVERFLOW, data & 0b0100_0000 > 0);
        self.status.set(CPUFlags::NEGATIVE, data & 0b1000_0000 > 0);
    }

    // Decrement Memory
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.update_zero_and_negative_flags(data.wrapping_sub(1));
        self.mem_write(addr, data.wrapping_sub(1));
    }

    // Decrement X
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Decrement Y
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    // Exclusive OR
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a ^= data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // JMP - jump
    fn jmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read_u16(addr);
        self.program_counter = value;
    }

    // Load Accumulator
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Load Accumulator X
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_x = value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Load Accumulator Y
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_y = value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // Transfer Accumulator to X
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Transfer Accumulator to Y
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // Transfer Stack Pointer to X
    fn tsx(&mut self) {
        self.register_x = self.register_s;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Transfer X to Accumulator
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Transfer Y to Accumulator
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Increment Memory
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.mem_write(addr, data.wrapping_add(1));
        self.update_zero_and_negative_flags(data.wrapping_add(1));
    }
    
    // Increment X
    fn inx(&mut self) {
        // Programming in overflow
        self.register_x = self.register_x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register_x);
    }

    // Increment Y
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn rotate_left(&mut self, mut data: u8) -> u8 {
        let new_carry = if data & 0x80 > 0 { 0b1 } else { 0b0 };
        data = (data << 1) | (if self.status.contains(CPUFlags::CARRY) { 0b1 } else { 0b0 });
        self.status.set(CPUFlags::CARRY, new_carry == 1);
        return data;
    }

    // Rotate Left
    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.rotate_left(self.mem_read(addr));
        self.mem_write(addr, data);
    }

    fn rotate_right(&mut self, mut data: u8) -> u8 {
        let new_carry = if data & 0x01 > 0 { 0b1 } else { 0b0 };
        data = (data >> 1) | (if self.status.contains(CPUFlags::CARRY) { 0x80 } else { 0x00 });
        self.status.set(CPUFlags::CARRY, new_carry == 1);
        return data;
    }

    // Rotate Right
    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.rotate_left(self.mem_read(addr));
        self.mem_write(addr, data);
    }

    // Subtract with Carry
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        // A - B - (1 - C) 
        // A + (-B) + C - 1
        // A + !B + 1 + C - 1
        // A + !B + C
        self.add_to_acc_with_carry(!data)
    }

    // Store Accumulator
    fn sta(&mut self, mode: &AddressingMode) {
        let target = self.get_operand_address(mode);
        self.mem_write(target, self.register_a);
    }

    // Store X
    fn stx(&mut self, mode: &AddressingMode) {
        let target = self.get_operand_address(mode);
        self.mem_write(target, self.register_x);
    }

    // Store Y
    fn sty(&mut self, mode: &AddressingMode) {
        let target = self.get_operand_address(mode);
        self.mem_write(target, self.register_y);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // If result=0... Set the Z (Zero) flag to 1
        self.status.set(CPUFlags::ZERO, result == 0);

        // If the 7th bit of result is set (i.e. negative number)
        // Set the N (negative) flag to 0 
        self.status.set(CPUFlags::NEGATIVE, result & 0b1000_0000 != 0);

    }

    fn calculate_branch_offset_clear(&mut self, condition: CPUFlags) -> u16 {
        if !self.status.contains(condition) {
            return self.mem_read(self.program_counter+1) as u16
        }
        return 0
    }

    fn calculate_branch_offset_set(&mut self, condition: CPUFlags) -> u16 {
        if self.status.contains(condition) {
            return self.mem_read(self.program_counter+1) as u16
        }
        return 0
    }

    fn add_to_acc_with_carry(&mut self, data: u8) {
        let mut result: u16 = self.register_a as u16 + data as u16;
        if self.status.contains(CPUFlags::CARRY) {
            result += 1;
        }

        self.status.set(CPUFlags::CARRY, result > 0xFF);

        let result = result as u8;
        // (M^result)&(N^result)&0x80 
        // If the sign of both inputs is different from result
        self.status.set(CPUFlags::OVERFLOW, (self.register_a ^ result) & (data ^ result) & 0x80 != 0);
      
        self.register_a = result as u8;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn compare(&mut self, register: u8, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        let compare = register - data;

        self.status.set(CPUFlags::CARRY, compare & 0b1000_0000 == 0);
        self.status.set(CPUFlags::NEGATIVE, compare & 0b1000_0000 > 0);
        self.status.set(CPUFlags::ZERO, compare == 0);
    }

    pub fn run(&mut self) {

        loop {
            let opcode = OPCODES_MAP.get(&self.mem_read(self.program_counter)).unwrap();
            self.program_counter += 1;

            match opcode.code {
                // BRK
                0x00 => return,

                // NOP
                0xEA => {}, // Do nothing

                // ADC
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.adc(&opcode.mode),

                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),

                // ASL ACCUMULATOR
                0x0A => self.asl_acc(),

                // ASL
                0x06 | 0x16 | 0x0E | 0x1E => self.asl(&opcode.mode),

                // BCC
                0x90 => self.program_counter += self.calculate_branch_offset_clear(CPUFlags::CARRY),         

                // BCS
                0xB0 => self.program_counter += self.calculate_branch_offset_set(CPUFlags::CARRY),

                // BEQ
                0xF0 => self.program_counter += self.calculate_branch_offset_set(CPUFlags::ZERO),

                // BMI
                0x30 => self.program_counter += self.calculate_branch_offset_set(CPUFlags::NEGATIVE),

                // BNE
                0xD0 => self.program_counter += self.calculate_branch_offset_clear(CPUFlags::ZERO),

                // BPL
                0x10 => self.program_counter += self.calculate_branch_offset_clear(CPUFlags::NEGATIVE),

                // BVC
                0x50 => self.program_counter += self.calculate_branch_offset_clear(CPUFlags::OVERFLOW),

                // BVS
                0x70 => self.program_counter += self.calculate_branch_offset_set(CPUFlags::OVERFLOW),

                // BIT
                0x24 | 0x2C => self.bit(&opcode.mode),

                // CLC
                0x18 => self.status.remove(CPUFlags::CARRY),

                // CLD
                0xD8 => self.status.remove(CPUFlags::DECIMAL),

                // CLI
                0x58 => self.status.remove(CPUFlags::INTERRUPT),

                // CLV
                0xB8 => self.status.remove(CPUFlags::OVERFLOW),

                // CMP
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => self.compare(self.register_a, &opcode.mode),

                // CPX
                0xE0 | 0xE4 | 0xEC => self.compare(self.register_x, &opcode.mode),

                // CPY
                0xC0 | 0xC4 | 0xCC => self.compare(self.register_y, &opcode.mode),

                // DEC
                0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(&opcode.mode),

                // DEX
                0xCA => self.dex(),

                // DEY       
                0x88 => self.dey(),

                // EOR
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51  => self.eor(&opcode.mode),

                // INC
                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&opcode.mode),

                // INX
                0xE8 => self.inx(),

                // INY
                0xC8 => self.iny(),

                // JMP
                0x4C | 0x6C => self.jmp(&opcode.mode),

                // JSR
                0x20 => {},

                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.lda(&opcode.mode),

                // LDX
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.ldx(&opcode.mode),

                // LDY
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.ldy(&opcode.mode),

                // LSR
                0x4A | 0x46 | 0x56 | 0x4E | 0x5E => {},

                // ORA
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {},

                // PHA
                0x48 => {},

                // PHP
                0x08 => {},

                // PLA
                0x68 => {},

                // ROL ACCUMULATOR
                0x2A => self.register_a = self.rotate_left(self.register_a),

                // ROL
                0x26 | 0x36 | 0x2E | 0x3E => self.rol(&opcode.mode),

                // ROR ACCUMULATOR
                0x6A => self.register_a = self.rotate_right(self.register_a),
                 
                // ROR
                0x66 | 0x76 | 0x6E | 0x7E => self.ror(&opcode.mode),

                // RTI
                0x40 => {},

                // RTS
                0x60 => {},

                // SBC
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.sbc(&opcode.mode),

                // SEC
                0x38 => self.status.insert(CPUFlags::CARRY),

                // SED
                0xF8 => self.status.insert(CPUFlags::DECIMAL),

                // SEI
                0x78 => self.status.insert(CPUFlags::INTERRUPT),

                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(&opcode.mode),

                // STX
                0x86 | 0x96 | 0x8E => self.stx(&opcode.mode),

                // STY
                0x84 | 0x94 | 0x8C => self.sty(&opcode.mode),

                // TAX
                0xAA => self.tax(),

                // TAY
                0xA8 => self.tay(),
                
                // TSX
                0xBA => self.tsx(),

                // TXA
                0x8A => self.txa(),

                // TXS
                0x9A  => self.register_s = self.register_x, // No flags needed to update

                // TYA
                0x98  => self.tya(),

                
                _ => todo!("")
            }
            self.program_counter += opcode.len as u16 - 1;
        }

    }

}


/*
Used the following link to test out the tests and view outputs
https://skilldrick.github.io/easy6502/
*/
#[cfg(test)]
mod test { 
    use super::*;
    
    #[test]
    fn test_set_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xF8, 0x78, 0x00]); // SEC SED SEI BRK
        assert!(cpu.status.contains(CPUFlags::CARRY));
        
        assert!(cpu.status.contains(CPUFlags::DECIMAL));
        assert!(cpu.status.contains(CPUFlags::INTERRUPT));
    }

    #[test]
    fn test_clear_flags() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xF8, 0x78, 0x18, 0xD8, 0x58, 0x00]); // SEC SED SEI CLC CLD CLI BRK
        
        assert!(!cpu.status.contains(CPUFlags::CARRY));
        assert!(!cpu.status.contains(CPUFlags::DECIMAL));
        assert!(!cpu.status.contains(CPUFlags::INTERRUPT));
    }


    #[test]
    fn test_adc_immediate_without_carry() {
        
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x69, 0x01, 0x00]); // ADC #01 BRK
        assert_eq!(cpu.register_a, 0x01); // Check if A = 1
        assert!(!cpu.status.contains(CPUFlags::ZERO)); // Check the Z flag is off
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE)); // Check the N flag is off
    }
    
    #[test]
    fn test_adc_immediate_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0x69, 0x01, 0x00]); // SEC ADC #01 BRK
        assert_eq!(cpu.register_a, 0x02); // Check if A = 2
    }

    #[test]
    fn test_adc_carry_and_overflow_flags() {
        
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x80, 0x69, 0x80, 0x00]); // LDA #80 ADC #80 BRK
        assert!(cpu.status.contains(CPUFlags::CARRY)); 
        
        assert!(cpu.status.contains(CPUFlags::OVERFLOW));
    }

    #[test]
    fn test_asl() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x42, 0x0A, 0x00]); // LDA #42 ASL A BRK
        assert!(cpu.register_a == 0x84);
    }

    #[test]
    fn test_asl_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x42, 0x85, 0x00, 0x06, 0x00, 0x00]); // LDA #42 STA $00 ASL $00 BRK
        assert!(cpu.mem_read(0x00) == 0x84);
    }

    #[test]
    fn test_asl_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x0A, 0x00]); // LDA #FF ASL BRK
        assert!(cpu.status.contains(CPUFlags::CARRY));
    }

    #[test]
    fn test_bit_clear_all() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x0F, 0x85, 0x00, 0x24, 0x00, 0x00]); // LDA #0F STA $00 BIT $00 BRK
        assert!(!cpu.status.contains(CPUFlags::ZERO));
        assert!(!cpu.status.contains(CPUFlags::OVERFLOW));
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE));
    }

    
    #[test]
    fn test_bit_set_all() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xF0, 0x85, 0x00, 0xa9, 0x0F, 0x24, 0x00, 0x00]); // LDA #F0 STA $00 LDA #0F BIT $00 BRK
        assert!(cpu.status.contains(CPUFlags::ZERO));
        assert!(cpu.status.contains(CPUFlags::OVERFLOW));
        assert!(cpu.status.contains(CPUFlags::NEGATIVE));
    }

    #[test]
    fn test_sbc_immediate_without_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xE9, 0x01, 0x00]); // SBC #01 BRK

        assert_eq!(cpu.register_a, 0xFE); // Check if A = -2
        assert!(!cpu.status.contains(CPUFlags::ZERO)); // Check the Z flag is off
        assert!(cpu.status.contains(CPUFlags::NEGATIVE)); // Check the N flag is on
    }
    
    #[test]
    fn test_sbc_immediate_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xE9, 0x00, 0x00]); // SEC SBC #00 BRK
        assert_eq!(cpu.register_a, 0x00); // Check if A = 0
    }

    #[test]
    fn test_and_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x29, 0x01, 0x00]); // Load 5 into acc and AND with 1
        assert_eq!(cpu.register_a, 0x01); // Check if A = 1
        assert!(!cpu.status.contains(CPUFlags::ZERO)); // Check the Z flag is off
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE)); // Check the N flag is off
    }
    
    #[test]
    fn test_and_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x29, 0x00, 0x00]); // Load 5 into acc and AND with 0
        assert!(cpu.status.contains(CPUFlags::ZERO)) // Check Z flag is on
    }

    #[test]
    fn test_and_from_memory() {
        
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x51);

        cpu.load_and_run(vec![0xa9, 0x55, 0x25, 0x10, 0x00]); // Load 5 into acc and AND with location in 
        assert!(cpu.register_a == 0x51) 
    }

    #[test]
    fn test_bcc() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x90, 0x02, 0x78, 0x00]); // BCC #02 SEI BRK
        
        assert!(!cpu.status.contains(CPUFlags::INTERRUPT));
    }

    #[test]
    fn test_cmp_greater_than() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x01, 0xc9, 0x00, 0x00]); // LDA #$01 CMP #$00 BRK
        assert!(cpu.status.contains(CPUFlags::CARRY));
        assert!(!cpu.status.contains(CPUFlags::ZERO));
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE));
    }

    #[test]
    fn test_cmp_equal_to() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xc9, 0x00, 0x00]); // CMP #$00 BRK
        assert!(cpu.status.contains(CPUFlags::CARRY));
        assert!(cpu.status.contains(CPUFlags::ZERO));
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_cmp_less_than() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0xc9, 0x00, 0x00]); // LDA #$-1 CMP #$00 BRK
        
        assert!(!cpu.status.contains(CPUFlags::CARRY));
        assert!(!cpu.status.contains(CPUFlags::ZERO));
        assert!(cpu.status.contains(CPUFlags::NEGATIVE));
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]); // Load in 5 into the Accumulator
        assert_eq!(cpu.register_a, 0x05); // Check if A = 5
        assert!(!cpu.status.contains(CPUFlags::ZERO)); // Check the Z flag is off
        assert!(!cpu.status.contains(CPUFlags::NEGATIVE)); // Check the N flag is off
    }
    
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]); // Load 0 into accumulator
        assert!(cpu.status.contains(CPUFlags::ZERO)) // Check Z flag is on
    }

    #[test]
    fn test_lda_from_memory() {
        
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]); // LDA from 0x10
        assert!(cpu.register_a == 0x55) 
    }

    #[test]
    fn test_sta() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x60;

        cpu.load(vec![0x85, 0x10, 0x00]); // STA to 0x10
        cpu.reset();
        cpu.register_a = 0x60;
        cpu.run();
        let mem = cpu.mem_read(0x10);
        assert_eq!(mem, 0x60) 
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xaa, 0x00]); // TAX
        cpu.reset();
        cpu.register_a = 5;
        cpu.run();
        assert_eq!(cpu.register_x, 5) // Check Z flag is on
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]); // Load C0 into A, transfer to X, then increment
    
        assert_eq!(cpu.register_x, 0xc1) // 0xc0 + 1 = 0xc1
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load(vec![0xe8, 0xe8, 0x00]); // Already at 0xff, +1 overflow to 0, +1 = 1
        
        cpu.reset();
        cpu.register_x = 0xff;
        cpu.run();
        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xFF, 0x2A, 0x00]); // LDA #$FF ROL BRK
        
        assert_eq!(cpu.register_a, 0xFE);
        assert!(cpu.status.contains(CPUFlags::CARRY));
    }

    #[test]
    fn test_rol_with_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x38, 0xa9, 0x7F, 0x2A, 0x00]); // SEC, LDA #$FF ROL BRK
        
        assert_eq!(cpu.register_a, 0xFF);
        assert!(!cpu.status.contains(CPUFlags::CARRY));
    }


}