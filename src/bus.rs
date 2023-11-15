use crate::{cpu::Memory, cartridge::Rom, ppu::ppu::PPU, input::joypad::Joypad};

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: PPU,
    joypad: Joypad,

    pub cycles: usize,
    gameloop_callback: Box<dyn FnMut(&PPU, &mut Joypad) + 'call>,
}

impl<'a> Bus<'a> {
    pub fn new<'call, F>(rom: Rom, gameloop_callback: F) -> Bus<'call> 
    where F: FnMut(&PPU, &mut Joypad) + 'call,
    {
        let ppu = PPU::new(rom.chr_rom, rom.screen_mirroring);
        let joypad = Joypad::new();
        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu,
            joypad,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
        }
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // Mirror
            addr = addr % 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        return self.ppu.nmi_interrupt.take();
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;

        let new_frame = self.ppu.tick(cycles * 3); // PPU clock is 3x faster than CPU clock

        if new_frame {
            (self.gameloop_callback)(&self.ppu, &mut self.joypad);
        }
    }
}

const RAM: u16 = 0x0000; 
const RAM_MIRRORS_END: u16 = 0x1FFF; 
// PPU Registers
const PPUCTRL: u16   = 0x2000; 
const PPUMASK: u16   = 0x2001; 
const PPUSTATUS: u16 = 0x2002; 
const OAMADDR: u16   = 0x2003; 
const OAMDATA: u16   = 0x2004; 
const PPUSCROLL: u16 = 0x2005; 
const PPUADDR: u16   = 0x2006; 
const PPUDATA: u16   = 0x2007; 
const OAMDMA: u16    = 0x4014; 
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl Memory for Bus<'_> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }

            PPUCTRL | PPUMASK | OAMADDR | PPUSCROLL | PPUADDR | OAMDMA => {
                //panic!("Attempting to read from write-only PPU Address {:X}", addr);
                0
            }

            PPUSTATUS => self.ppu.read_status(),
            PPUDATA => self.ppu.read_data(),
            OAMDATA => self.ppu.read_oam_data(),

            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }

            0x4000..=0x4015 => {
                //ignore APU 
                0
            }

            0x4016 => {
                self.joypad.read()
            }

            0x4017 => {
                // ignore joypad 2
                0
            }


            0x8000..=0xFFFF => self.read_prg_rom(addr),

            _ => {
                println!("Unknown memory access at {:X}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM ..= RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }

            PPUCTRL => self.ppu.write_to_control(data),
            PPUMASK => self.ppu.write_to_mask(data),
            PPUSTATUS => {},//panic!("Attempting to write {:X} to PPU Status", data),
            PPUADDR => self.ppu.write_to_ppu_addr(data),
            PPUSCROLL => self.ppu.write_to_scroll(data),

            OAMADDR => self.ppu.write_to_oam_addr(data),
            OAMDATA => self.ppu.write_to_oam_data(data),
            PPUDATA => self.ppu.write_to_data(data),

            0x2008 ..= PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            
            OAMDMA => {
                let start =( (data as u16) << 8) as usize;
                let end = ((data as u16) << 8 | 0xFF) as usize;
                self.ppu.oam_dma(&self.cpu_vram[start..end]);
            }

            0x4000..=0x4013 | 0x4015 => {
                //ignore APU 
            
            }

            0x4016 => {
                self.joypad.write(data);
            }

            0x4017 => {
                // ignore joypad 2
                
            }

            0x8000..=0xFFFF => {
                panic!("Attempting to write to Cartridge ROM space")
            }

            _ => {
                println!("Unknown memory access at {:X}", addr);
            }
        }
    }
}
