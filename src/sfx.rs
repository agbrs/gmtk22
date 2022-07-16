use core::cmp::Ordering;

use agb::fixnum::{num, Num};
use agb::sound::mixer::{ChannelId, Mixer, SoundChannel};
use agb::{include_wav, rng};

const DICE_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/SingleRoll_1.wav"),
    include_wav!("sfx/SingleRoll_2.wav"),
    include_wav!("sfx/SingleRoll_3.wav"),
    include_wav!("sfx/SingleRoll_4.wav"),
    include_wav!("sfx/SingleRoll_5.wav"),
];

const MULTI_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/MultiRoll_1.wav"),
    include_wav!("sfx/MultiRoll_2.wav"),
    include_wav!("sfx/MultiRoll_3.wav"),
    include_wav!("sfx/MultiRoll_4.wav"),
    include_wav!("sfx/MultiRoll_5.wav"),
];

const BATTLE_BGM: &[u8] = include_wav!("sfx/BGM_Fight.wav");
const MENU_BGM: &[u8] = include_wav!("sfx/BGM_Menu.wav");

pub struct Sfx<'a> {
    mixer: &'a mut Mixer,
    cross_fade: Num<i16, 4>,
    target_cross_fade: Num<i16, 4>,

    customise_channel: ChannelId,
    battle_channel: ChannelId,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer) -> Self {
        let mut battle_music = SoundChannel::new_high_priority(BATTLE_BGM);
        battle_music.should_loop().stereo().volume(0);
        let battle_channel = mixer.play_sound(battle_music).unwrap();

        let mut menu_music = SoundChannel::new_high_priority(MENU_BGM);
        menu_music.should_loop().stereo().volume(1);
        let menu_channel = mixer.play_sound(menu_music).unwrap();

        Self {
            mixer,
            cross_fade: num!(1.0),
            target_cross_fade: num!(1.0),

            customise_channel: menu_channel,
            battle_channel,
        }
    }

    pub fn frame(&mut self) {
        match self.target_cross_fade.cmp(&self.cross_fade) {
            Ordering::Less => {
                self.cross_fade -= num!(0.05);
            }
            Ordering::Greater => {
                self.cross_fade += num!(0.05);
            }
            _ => {}
        }

        self.mixer
            .channel(&self.customise_channel)
            .unwrap()
            .volume(self.cross_fade);
        self.mixer
            .channel(&self.battle_channel)
            .unwrap()
            .volume(num!(1.) - self.cross_fade);

        self.mixer.frame();
    }

    pub fn battle(&mut self) {
        self.target_cross_fade = num!(0.0);
    }

    pub fn customise(&mut self) {
        self.target_cross_fade = num!(1.0);
    }

    pub fn roll(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(DICE_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(DICE_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }

    pub fn roll_multi(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(MULTI_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(MULTI_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }
}
