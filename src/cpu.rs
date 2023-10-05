use crate::opcodes::OPCODES_MAP;
pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    /*
    NVss DIZC
    7_______0

    N - Negative
    V - Overflow
    ss - B flag, doesn't affect CPU operation, 1 by default
    D - Decimal
    I - Interrupt Signal
    Z - Zero 
    C - Carry
     */
    pub status: u8,
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
            status: 0,
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
        self.status = 0;
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

    // Logical AND
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_a &= self.mem_read(addr);
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Load Accumulator
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // Transfer Accumulator to X
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // Increment X
    fn inx(&mut self) {
        // Programming in overflow
        self.register_x = self.register_x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.register_x);
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let target = self.get_operand_address(mode);
        self.mem_write(target, self.register_a);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        // If result=0... Set the Z (Zero) flag to 1
        if result == 0 {
            self.status = self.status | 0b0000_0010;
        } else {
            // If not, then set it to 0
            self.status = self.status & 0b1111_1101;
        }

        // If the 7th bit of result is set (i.e. negative number)
        // Set the N (negative) flag to 0 
        if result & 0b1000_0000 != 0 {
            self.status = self.status | 0b1000_0000; 
        } else {
            self.status = self.status & 0b0111_1111;
        }
    }

    pub fn run(&mut self) {

        loop {
            let opcode = *&OPCODES_MAP.get(&self.mem_read(self.program_counter)).unwrap();
            self.program_counter += 1;

            match opcode.code {
                
                // AND
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.mode);
                }
                
                // LDA
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.mode);
                }
                
                // STA
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                0xAA => self.tax(), // TAX
                0xE8 => self.inx(), // INX
                0x00 => return, // BRK

                _ => todo!("")
            }
            self.program_counter += opcode.len as u16 - 1;
        }

    }

}


#[cfg(test)]
mod test { 
    use super::*;
    
    #[test]
    fn test_and_immediate() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x29, 0x01, 0x00]); // Load 5 into acc and AND with 1
        assert_eq!(cpu.register_a, 0x01); // Check if A = 1
        assert!(cpu.status & 0b0000_0010 == 0b00); // Check the Z flag is off
        assert!(cpu.status & 0b1000_0000 == 0); // Check the N flag is off
    }
    
    #[test]
    fn test_and_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x29, 0x00, 0x00]); // Load 5 into acc and AND with 0
        assert!(cpu.status & 0b0000_0010 == 0b10) // Check Z flag is on
    }

    #[test]
    fn test_and_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x51);

        cpu.load_and_run(vec![0xa9, 0x55, 0x25, 0x10, 0x00]); // Load 5 into acc and AND with location in 
        assert!(cpu.register_a == 0x51) 
    }


    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]); // Load in 5 into the Accumulator
        assert_eq!(cpu.register_a, 0x05); // Check if A = 5
        assert!(cpu.status & 0b0000_0010 == 0b00); // Check the Z flag is off
        assert!(cpu.status & 0b1000_0000 == 0); // Check the N flag is off
    }
    
    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]); // Load 0 into accumulator
        assert!(cpu.status & 0b0000_0010 == 0b10) // Check Z flag is on
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
}