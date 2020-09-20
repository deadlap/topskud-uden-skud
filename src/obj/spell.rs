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

use super::{Object, player::Player, projectile::{Projectile, ProjectileMaker}, explosion::{Explosion, ExplosionMaker}};

pub enum ObjMaker<'a> {
    Explosion(ExplosionMaker<'a>),
    Projectile(ProjectileMaker<'a>)
}

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
    pub pattern: Vec<f32>,
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

    pub fn can_cast_spell(&mut self) -> bool {
        self.cooldown_time_left == 0.
    }

    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()>{
        if self.cooldown_time_left > DELTA {
            self.cooldown_time_left -= DELTA
        } else {
            self.cooldown_time_left = 0.;
        }
        Ok(())
    }

    pub fn cast(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<ObjMaker<'a>>> {
        // if self.cooldown_time_left == 0. {
        mplayer.play(ctx, "throw")?;
        // self.cooldown_time_left = self.spell.cooldown_time;
        // use CastType::*;
        match self.spell.cast_type {
            CastType::Explosion => Ok(Some(ObjMaker::Explosion(ExplosionMaker(self.spell, &self.spell.pattern)))),
            CastType::Projectile => Ok(Some(ObjMaker::Projectile(ProjectileMaker(self.spell, &self.spell.pattern)))),
        }
        // } else {
        //     Ok(None)
        // }
    }
}