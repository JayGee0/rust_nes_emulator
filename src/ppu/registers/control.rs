bitflags! {
    /*
    VPHB SINN
    7_______0

    V - NMI (Non-Masking Interrupt), set whether occurred 
    P - PPU Master/Slave select (0 read, 1 write)
    H - Sprite size mode (0: 8x8, 1: 8x16)
    B - Background pattern table address mode (0: $0000, 1: $1000)

    S - Sprite pattern table address for 8x8 (0: $0000, 1: $10000, ignore if in 8x16 mode)
    I - VRAM address increment mode per CPU read/write of PPUDATA (0: add 1, 1: add 32, going down)
    NN - Base nametable address (00: $2000, 01: $2400, 10: $2800, 11: $2C00)
    */
    pub struct ControlRegister: u8 {
        const GENERATE_NMI            = 0b10000000;
        const MASTER_SLAVE_SELECT     = 0b01000000;
        const SPRITE_SIZE             = 0b00100000;
        const BACKROUND_PATTERN_ADDR  = 0b00010000;
        const SPRITE_PATTERN_ADDR     = 0b00001000;
        const VRAM_ADD_INCREMENT      = 0b00000100;
        const NAMETABLE2              = 0b00000010;
        const NAMETABLE1              = 0b00000001;
    }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0b00000000)
    }

    pub fn vram_addr_increment(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}