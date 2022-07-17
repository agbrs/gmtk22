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

const MENU_BGM: &[u8] = include_wav!("sfx/BGM_Fight.wav");
const BATTLE_BGM: &[u8] = include_wav!("sfx/BGM_Menu.wav");

const SHOOT: &[u8] = include_wav!("sfx/shoot.wav");
const SHOT_HIT: &[u8] = include_wav!("sfx/shot_hit.wav");
const SHIP_EXPLODE: &[u8] = include_wav!("sfx/ship_explode.wav");
const MOVE_CURSOR: &[u8] = include_wav!("sfx/move_cursor.wav");
const SELECT: &[u8] = include_wav!("sfx/select.wav");
const BACK: &[u8] = include_wav!("sfx/back.wav");
const ACCEPT: &[u8] = include_wav!("sfx/accept.wav");
const SHIELD_DOWN: &[u8] = include_wav!("sfx/shield_down.wav");
const SHIELD_UP: &[u8] = include_wav!("sfx/shield_up.wav");
const SHIELD_DEFEND: &[u8] = include_wav!("sfx/shield_defend.wav");

const MAX_CROSSFADE_FRAMES: i16 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BattleOrMenu {
    Battle,
    Menu,
}

pub struct Sfx<'a> {
    mixer: &'a mut Mixer,
    frames_for_cross_fade: i16,
    state: BattleOrMenu,

    customise_channel: ChannelId,
    battle_channel: ChannelId,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer) -> Self {
        let mut battle_music = SoundChannel::new_high_priority(BATTLE_BGM);
        battle_music.should_loop().playback(num!(1.)).volume(0);
        let battle_channel = mixer.play_sound(battle_music).unwrap();

        let mut menu_music = SoundChannel::new_high_priority(MENU_BGM);
        menu_music.should_loop().playback(num!(1.)).volume(1);
        let menu_channel = mixer.play_sound(menu_music).unwrap();

        Self {
            mixer,
            frames_for_cross_fade: MAX_CROSSFADE_FRAMES,
            state: BattleOrMenu::Menu,

            customise_channel: menu_channel,
            battle_channel,
        }
    }

    pub fn frame(&mut self) {
        self.frames_for_cross_fade = (self.frames_for_cross_fade + 1).min(MAX_CROSSFADE_FRAMES);

        let active_volume = Num::new(self.frames_for_cross_fade) / MAX_CROSSFADE_FRAMES;
        let (battle_volume, menu_volume) = match self.state {
            BattleOrMenu::Battle => (num!(1.) - active_volume, active_volume),
            BattleOrMenu::Menu => (active_volume, num!(1.) - active_volume),
        };

        self.mixer
            .channel(&self.customise_channel)
            .unwrap()
            .volume(menu_volume);
        self.mixer
            .channel(&self.battle_channel)
            .unwrap()
            .volume(battle_volume);

        self.mixer.frame();
    }

    pub fn battle(&mut self) {
        if self.state == BattleOrMenu::Battle {
            return;
        }

        self.state = BattleOrMenu::Battle;
        self.frames_for_cross_fade = 0;
    }

    pub fn customise(&mut self) {
        if self.state == BattleOrMenu::Menu {
            return;
        }

        self.state = BattleOrMenu::Menu;
        self.frames_for_cross_fade = 0;
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

    pub fn shoot(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHOOT));
    }

    pub fn shot_hit(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHOT_HIT));
    }

    pub fn ship_explode(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIP_EXPLODE));
    }

    pub fn move_cursor(&mut self) {
        let mut channel = SoundChannel::new(MOVE_CURSOR);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn select(&mut self) {
        let mut channel = SoundChannel::new(SELECT);
        channel.volume(num!(0.75));

        self.mixer.play_sound(channel);
    }

    pub fn back(&mut self) {
        let mut channel = SoundChannel::new(BACK);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn accept(&mut self) {
        let mut channel = SoundChannel::new(ACCEPT);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn shield_down(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIELD_DOWN));
    }

    pub fn shield_up(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIELD_UP));
    }

    pub fn shield_defend(&mut self) {
        let mut channel = SoundChannel::new(SHIELD_DEFEND);
        channel.volume(num!(0.5));
        self.mixer.play_sound(channel);
    }
}
