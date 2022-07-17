use agb::{
    display::tiled::{RegularMap, TileFormat, TileSet, TileSetting, VRamManager},
    include_gfx, rng,
};

include_gfx!("gfx/stars.toml");

pub fn load_palettes(vram: &mut VRamManager) {
    vram.set_background_palettes(&[
        stars::stars.palettes[0].clone(),
        crate::customise::DESCRIPTIONS_1_PALETTE.clone(),
        crate::customise::DESCRIPTIONS_2_PALETTE.clone(),
    ]);
}

// Expects a 64x32 map
fn create_background_map(map: &mut RegularMap, vram: &mut VRamManager, stars_tileset: &TileSet) {
    for x in 0..64u16 {
        for y in 0..32u16 {
            let blank = rng::gen().rem_euclid(32) < 30;

            let tile_id = if blank {
                (1 << 10) - 1
            } else {
                rng::gen().rem_euclid(64) as u16
            };
            let tile_setting = TileSetting::new(tile_id, false, false, 0);

            map.set_tile(vram, (x, y).into(), stars_tileset, tile_setting);
        }
    }

    map.set_scroll_pos((0u16, rng::gen().rem_euclid(8) as u16).into());
}

pub struct StarBackground<'a> {
    background1: &'a mut RegularMap,
    background2: &'a mut RegularMap,

    background1_timer: u32,
    background2_timer: u32,
}

impl<'a> StarBackground<'a> {
    pub fn new(
        background1: &'a mut RegularMap,
        background2: &'a mut RegularMap,
        vram: &'_ mut VRamManager,
    ) -> Self {
        let stars_tileset = TileSet::new(stars::stars.tiles, TileFormat::FourBpp);
        create_background_map(background1, vram, &stars_tileset);
        create_background_map(background2, vram, &stars_tileset);

        Self {
            background1,
            background2,

            background1_timer: 0,
            background2_timer: 0,
        }
    }

    pub fn update(&mut self) {
        if self.background1_timer == 0 {
            self.background1
                .set_scroll_pos(self.background1.scroll_pos() + (1u16, 0).into());
            self.background1_timer = 2;
        }

        if self.background2_timer == 0 {
            self.background2
                .set_scroll_pos(self.background2.scroll_pos() + (1u16, 0).into());
            self.background2_timer = 3;
        }

        self.background1_timer -= 1;
        self.background2_timer -= 1;
    }

    pub fn commit(&mut self, vram: &mut VRamManager) {
        self.background1.commit(vram);
        self.background2.commit(vram);
    }
}
