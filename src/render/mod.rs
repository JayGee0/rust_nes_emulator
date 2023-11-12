pub mod frame;
pub mod palette;

use crate::ppu::ppu::PPU;
use frame::Frame;

pub fn render(ppu: &PPU, frame: &mut Frame) {
    let bank = ppu.control.background_pattern_addr();

    for i in 0..0x03C0 {
        // just for now, lets use the first nametable
        let tile = ppu.vram[i] as u16;
        let tile_x = i % 32;
        let tile_y = i / 32;
        let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
        let palette = bg_pallette(ppu, tile_x, tile_y);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[palette[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("Palette selection out of bounds for background"),
                };
                frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
            }
        }
    }

    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_i = ppu.oam_data[i + 1] as u16;
        let tile_x = ppu.oam_data[i + 3] as usize;
        let tile_y = ppu.oam_data[i] as usize;

        let flip_vertical = ppu.oam_data[i + 2] >> 7 & 1 == 1;
        let flip_horizontal = ppu.oam_data[i + 2] >> 6 & 1 == 1;
        let pallette_i = ppu.oam_data[i + 2] & 0b11;
        let sprite_palette = sprite_palette(ppu, pallette_i);

        let bank: u16 = ppu.control.sprite_pattern_addr();

        let tile =
            &ppu.chr_rom[(bank + tile_i * 16) as usize..=(bank + tile_i * 16 + 15) as usize];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => continue, // skip coloring the pixel
                    1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
                    _ => panic!("Palette selection out of bounds for sprite"),
                };
                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                }
            }
        }
    }
}

pub fn bg_pallette(ppu: &PPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_i = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x3c0 + attr_table_i];

    let pallet_i = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        (_, _) => panic!("should not happen"),
    };

    let start: usize = 1 + (pallet_i as usize) * 4;
    // Default
    [
        ppu.palette[0],
        ppu.palette[start],
        ppu.palette[start + 1],
        ppu.palette[start + 2],
    ]
}

pub fn sprite_palette(ppu: &PPU, palette_i: u8) -> [u8; 4] {
    let start = 0x11 + (palette_i * 4) as usize;
    [
        0,
        ppu.palette[start],
        ppu.palette[start + 1],
        ppu.palette[start + 2],
    ]
}