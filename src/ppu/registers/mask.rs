bitflags! {
    /* 
    BGRs bMmG
    7_______0

    B - Emphasise Blue
    G - Emphasise Green
    R - Emphasise Red
    s - Show sprites

    b - Show background
    M - Show sprites in leftmost 8 pixels of the screen
    m - Show background in leftmost 8 pixels of the screen
    G - Greyscale 
    */
    pub struct MaskRegister: u8 {
        const EMPH_BLUE              = 0b10000000;
        const EMPH_GREEN             = 0b01000000;
        const EMPH_RED               = 0b00100000;
        const SHOW_SPRITES           = 0b00010000;
        const SHOW_BACKGROUND        = 0b00001000;
        const SHOW_SPRITES_LEFT      = 0b00000100;
        const SHOW_BACKGROUND_LEFT   = 0b00000010;
        const GREYSCALE              = 0b00000001;
    }

}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister::from_bits_truncate(0x00)
    }

    pub fn disable_rendering(&mut self) {
        self.set(MaskRegister::SHOW_SPRITES, false);
        self.set(MaskRegister::SHOW_BACKGROUND, false);
    }

    pub fn show_sprites(&self) -> bool {
        self.contains(MaskRegister::SHOW_SPRITES)
    } 
    
    pub fn show_background(&self) -> bool {
        self.contains(MaskRegister::SHOW_BACKGROUND)
    }

    pub fn is_greyscale(&self) -> bool {
        self.contains(MaskRegister::GREYSCALE)
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}