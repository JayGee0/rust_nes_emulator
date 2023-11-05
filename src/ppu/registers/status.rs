bitflags! {
    /* 
    VSO. ....
    7_______0

    V - Vertical blank has started
    S - Sprite 0 hit. When nonzero pixel of sprite 0 overlaps with nonzero background pixel
    O - Sprite overflow (>8 sprites appear)
    . - Open space
    */
    pub struct StatusRegister: u8 {
        const VERTICAL_BLANK         = 0b10000000;
        const SPRITE_0_HIT           = 0b01000000;
        const SPRITE_OVERFLOW        = 0b00100000;
        const UNUSED_0               = 0b00010000;
        const UNUSED_1               = 0b00001000;
        const UNUSED_2               = 0b00000100;
        const UNUSED_3               = 0b00000010;
        const UNUSED_4               = 0b00000001;
    }

}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::from_bits_truncate(0x00)
    }

    pub fn sprite_overflow(&self) -> bool {
        self.contains(StatusRegister::SPRITE_OVERFLOW)
    }

    pub fn set_sprite_overflow(&mut self, value: bool) {
        self.set(StatusRegister::SPRITE_OVERFLOW, value);
    }

    pub fn sprite_0_hit(&self) -> bool {
        self.contains(StatusRegister::SPRITE_0_HIT)
    }

    pub fn set_sprite_0_hit(&mut self, value: bool) {
        self.set(StatusRegister::SPRITE_0_HIT, value);
    }

    pub fn in_vertical_blank(&self) -> bool {
        self.contains(StatusRegister::VERTICAL_BLANK)
    }

    pub fn set_vertical_blank(&mut self, value: bool) {
        self.set(StatusRegister::VERTICAL_BLANK, value);
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}