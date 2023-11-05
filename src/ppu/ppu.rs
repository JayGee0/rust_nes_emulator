use crate::cartridge::Mirroring;

use super::registers::{address::AddrRegister, control::ControlRegister, mask::MaskRegister, status::StatusRegister, scroll::ScrollRegister};


pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub oam_addr: u8,
    pub mirroring: Mirroring,
    pub internal_data_buffer: u8,

    // REGISTERS
    // =====================
    pub addr: AddrRegister,
    pub control: ControlRegister,
    pub mask: MaskRegister,
    pub status: StatusRegister,
    pub scroll: ScrollRegister,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        PPU {
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            oam_addr: 0,
            mirroring,
            internal_data_buffer: 0,
            addr: AddrRegister::new(),
            control: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            scroll: ScrollRegister::new(),
        }
    }

    fn increment_vram_addr(&mut self) {
        self.addr.increment(self.control.vram_addr_increment());
    }

    pub fn read_oam_data(&mut self) -> u8 {
        self.oam_data[self.oam_addr as usize]
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.chr_rom[addr as usize];
                result
            },
            0x2000..=0x2FFF => {
                let result = self.internal_data_buffer;
                self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            },
            0x3000..=0x3EFF => panic!("Not expected to use addr space 0x3000..0x3EFF, requested = {} ", addr),
            0x3F00..=0x3FFF => self.palette[(addr - 0x3F00) as usize],
            _ => panic!("Unexpected access to {}", addr),
        }
    }

    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // 0x2EFF, mirror down the 0x3000-0x3EFF to 0x2000-0x2EFF
        let vram_index = mirrored_vram - 0x2000; // To vram vector
        let name_table = vram_index / 0x400; // To nametable index

        match(&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAL, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAL, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn write_to_control(&mut self, value: u8) {
        self.control.update(value);
    }

    pub fn write_to_mask(&mut self, value: u8) {
        self.mask.update(value);
    }

    pub fn write_to_scroll(&mut self, value: u8) {
        self.scroll.write(value);
    }

    pub fn read_status(&self) -> u8 {
        self.status.bits()
    }

    pub fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_addr = value;
    }

    pub fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_addr as usize] = value;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_to_data(&mut self, value: u8) {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            0x0000..=0x1FFF => panic!("Attempting to write to chr rom space {}", addr),
            0x2000..=0x2FFF => self.vram[self.mirror_vram_addr(addr) as usize] = value,
            0x3000..=0x3EFF => panic!("Not expected to use addr space 0x3000..0x3EFF, requested = {} ", addr),
            0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
                let add_mirror = addr - 0x10;
                self.palette[self.mirror_vram_addr(add_mirror - 0x3F00) as usize] = value;
            }

            0x3F00..=0x3FFF => self.palette[(addr - 0x3F00) as usize] = value,
            _ => panic!("Unexpected access to {}", addr),
        }


    }

    pub fn oam_dma(&mut self, data: &[u8]) {
        self.oam_data.copy_from_slice(data);
    }
}
