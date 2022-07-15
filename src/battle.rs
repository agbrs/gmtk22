use crate::{Agb, PlayerDice};

pub(crate) fn battle_screen(agb: &mut Agb, player_dice: PlayerDice) {
    loop {
        agb.vblank.wait_for_vblank();
        agb.obj.commit();
    }
}
