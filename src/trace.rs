use std::collections::HashMap;

use crate::{CPU, opcodes, cpu::{Memory, AddressingMode}};

pub fn trace(cpu: &mut CPU) -> String {
    let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

    let current_PC = cpu.program_counter;

    // Instruction Opcode e.g. A2 01
    let code = cpu.mem_read(current_PC);
    let opcode = opcodes.get(&code).unwrap();

    let (mem_addr, mem_val) = match opcode.mode { // mem_addr = real address indexed, mem_val = value retrieved from memory
        AddressingMode::Immediate | AddressingMode::NoneAddressing => (0,0),
        _ => {
            let addr = cpu.get_operand_address_from_base(&opcode.mode, current_PC + 1);
            let mem = cpu.mem_read(addr);
            (addr, mem)
        }
    };

    // Get mode to determine how many to to print out
    let mut instruction = vec![code];


    let address_string
     = match opcode.len {
        1 => {
            match opcode.code {
                0x4A | 0x0A | 0x2A | 0x6A => format!("A"),
                _ => format!("")
            }
        }

        2 => {
            let byte_val = cpu.mem_read(current_PC+1); // Value held at the next byte
            instruction.push(byte_val);
            
            match opcode.mode {
                AddressingMode::Immediate => 
                    format!("#${:02X}", byte_val),
                AddressingMode::ZeroPage => 
                    format!("${:02X} = {:02X}", byte_val, mem_val),
                AddressingMode::ZeroPage_X => 
                    format!("${:02X},X @ {:02X} = {:02X}", byte_val, mem_addr, mem_val),
                AddressingMode::ZeroPage_Y => 
                    format!("${:02X},Y @ {:02X} = {:02X}", byte_val, mem_addr, mem_val),
                // STA ($80,X) @ 80 = 0200 = 5A
                AddressingMode::Indirect_X => 
                    format!("(${:02X},X) @ {:02X} = {:04X} = {:02X}", byte_val, cpu.register_x.wrapping_add(byte_val), mem_addr, mem_val),
                // LDA ($89),Y = 0300 @ 0300 = 89
                AddressingMode::Indirect_Y => 
                    format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X}", byte_val, mem_addr.wrapping_sub(cpu.register_y as u16), mem_addr, mem_val),
                
                AddressingMode::NoneAddressing => // Branch Instructions
                    format!("${:04X}", (current_PC as usize + 2).wrapping_add((byte_val as i8) as usize)),

                _ => panic!("Unexpected Addressing Mode at opcode.len = 2"),
            }
        },
        3 => {
            let lo_byte_val = cpu.mem_read(current_PC+1);
            let hi_byte_val = cpu.mem_read(current_PC+2);
            instruction.push(lo_byte_val);
            instruction.push(hi_byte_val);

            let byte_val = (hi_byte_val as u16) << 8 | (lo_byte_val as u16);

            match opcode.mode {
                AddressingMode::Absolute => {
                    if opcode.code == 0x4C || opcode.code == 0x20 { // JMP direct
                        format!("${:04X}", byte_val)
                    } else {
                        format!("${:04X} = {:02X}", byte_val, mem_val)
                    }
                },
                AddressingMode::Absolute_X => format!("${:04X},X @ {:04X} = {:02X}", byte_val, mem_addr, mem_val),
                AddressingMode::Absolute_Y => format!("${:04X},Y @ {:04X} = {:02X}", byte_val, mem_addr, mem_val),
                AddressingMode::NoneAddressing => {
                    if opcode.code == 0x6C {
                        format!("(${:04X}) = {:04X}", byte_val, cpu.calculate_jmp_indirect_bug(byte_val))// JMP indirect
                    } else {
                        format!("(${:04X})", byte_val)
                    }
                },

                _ => panic!("NO")
            }

        },
        _ => {
            format!("")
        }
    };
    
    let instruction_string = instruction
        .iter()
        .map(|z| format!("{:02X}", z))
        .collect::<Vec<String>>()
        .join(" ");

    let op_string = format!("{:04X}  {:8} {: >4} {}", current_PC, instruction_string, opcode.mnemonic, address_string);
    let status_string = format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} TICKS:{:}", 
                                    cpu.register_a, cpu.register_x, cpu.register_y, cpu.status.bits(), cpu.register_s, cpu.bus.cycles);

    return format!("{:47} {:24}", op_string, status_string);
}



#[cfg(test)]
mod test {
   use crate::cpu::Memory;
   use super::*;
   use crate::bus::Bus;
   use crate::cartridge::test::test_rom;

   #[test]
   fn test_format_trace() {
       let mut bus = Bus::new(test_rom(), |_|{});
       bus.mem_write(100, 0xa2);
       bus.mem_write(101, 0x01);
       bus.mem_write(102, 0xca);
       bus.mem_write(103, 0x88);
       bus.mem_write(104, 0x00);

       let mut cpu = CPU::new(bus);
       cpu.program_counter = 0x64;
       cpu.register_a = 1;
       cpu.register_x = 2;
       cpu.register_y = 3;
       let mut result: Vec<String> = vec![];
       cpu.run_with_callback(|cpu| {
           result.push(trace(cpu));
       });
       assert_eq!(
           "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
           result[0]
       );
       assert_eq!(
           "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
           result[1]
       );
       assert_eq!(
           "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
           result[2]
       );
   }

   #[test]
   fn test_format_mem_access() {
       let mut bus = Bus::new(test_rom(), |_|{});
       // ORA ($33), Y
       bus.mem_write(100, 0x11);
       bus.mem_write(101, 0x33);


       //data
       bus.mem_write(0x33, 00);
       bus.mem_write(0x34, 04);

       //target cell
       bus.mem_write(0x400, 0xAA);

       let mut cpu = CPU::new(bus);
       cpu.program_counter = 0x64;
       cpu.register_y = 0;
       let mut result: Vec<String> = vec![];
       cpu.run_with_callback(|cpu| {
           result.push(trace(cpu));
       });
       assert_eq!(
           "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
           result[0]
       );
   }
}

