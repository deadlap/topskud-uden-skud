use std::num::NonZeroU16;
use std::fmt::{self, Display};

use crate::{
    util::{Point2, angle_to_vec, sstr, add_sstr, Sstr},
    game::{DELTA, world::{World}},
    io::{
        snd::MediaPlayer,
        tex::{PosText, Assets},
    },
};
use ggez::{Context, GameResult};

use super::{Object, player::Player, projectile::Projectile, explosion::{Explosion, ExplosionMaker}};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Water,
    Fire,
    Ice,
    Electric,
    // Earth,
}

impl Element {
    pub fn get_spr(self) -> &'static str {
        use self::Element::*;
        match self {
            Water => "spells/elements/water",
            Fire => "spells/elements/fire",
            Ice => "spells/elements/ice",
            Electric => "spells/elements/electric",
            // Earth => "spells/elements//earth",
        }
    }

}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CastType {
    Projectile,
    Explosion,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SpellSlot {
    Element,
    // SpellType,
}

#[derive(Debug, Clone)]
pub struct Spell {
    pub id: Sstr,
    pub name: Sstr,
    pub cast_name: Sstr,
    pub element_type: Vec<Element>,
    pub energy_cost: f32,
    pub cast_type: CastType,
    pub cooldown_time: f32,
    pub charge_time: f32,
    pub cast_snd: Sstr,
    pub charge_snd: Sstr,
    pub spell_range: f32,
    // pub hands_sprite: Sstr,
}

mod consts;
pub use self::consts::*;

#[derive(Debug, Copy, Clone)]
pub struct SpellInstance<'a> {
    pub charged_time: f32,
    pub cooldown_time_left: f32,
    pub spell: &'a Spell,
}

impl Spell {
    pub fn make_instance(&self) -> SpellInstance<'_> {
        SpellInstance {
            charged_time: 0.,
            cooldown_time_left: 0.,
            spell: self,
        }
    }
}
impl<'a> SpellInstance<'a>{

    pub fn update(){

    }

    pub fn cast(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) {
        
    }
    
    pub fn cast_explosion(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<ExplosionMaker<'a>>> {
        // if player.energy > self.spell.energy_cost {
        mplayer.play(ctx, "throw")?;
        Ok(Some(ExplosionMaker(self.spell)))
        // } else {
        //     Ok(None)
        // }
    }
}