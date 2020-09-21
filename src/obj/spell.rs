use std::fmt::{self, Display};

use crate::{
    util::{Sstr},
    game::DELTA,
    io::{
        snd::MediaPlayer,
        tex::{PosText, Assets},
    },
};
use ggez::{Context, GameResult};

use super::{projectile::{ProjectileMaker}, explosion::{ExplosionMaker}};

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
    // Steel,
}

impl Element {
    pub fn get_spr(self) -> &'static str {
        use self::Element::*;
        match self {
            Water => "spells/elements/water",
            Fire => "spells/elements/fire",
            Ice => "spells/elements/ice",
            Electric => "spells/elements/electric",
            // Earth => "spells/elements/earth",
            // Steel => "spells/elements/steel",
        }
    }
    pub fn get_from_str(elem: &str) -> Option<Element> {
        use self::Element::*;
        let elem = elem.to_lowercase();
        match &*elem {
            "water" => Some(Water),
            "fire" => Some(Fire), 
            "ice" => Some(Ice),
            "electric" => Some(Electric),
            // "earth" => Some(Earth),
            // "steel" => Some(Steel),     
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum SpellType {
    Projectile,
    Explosion,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ChargeUpType {
    Damage,
    Range,
    Speed,
    Degrees,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum CastType {
    EndOfCharge,
    ChargeUp {
        charge_type: Vec<ChargeUpType>,
    },
    WhileCharging {
        frequency: f32,
    },
}
impl Default for CastType {
    fn default() -> Self{
        CastType::EndOfCharge
    }
}
impl CastType {
    #[inline]
    pub fn cast_while_charging(self) -> bool {
        if let CastType::WhileCharging{..} = self {
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn is_charge_up(self) -> bool {
        if let CastType::ChargeUp{..} = self {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spell {
    pub id: Sstr,
    pub name: Sstr,
    pub cast_name: Sstr,
    pub element_type: Vec<Element>,
    pub energy_cost: f32,
    pub spell_type: SpellType,
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
    pub ratio: f32,
    pub being_charged: bool,
    pub spell: &'a Spell,
}

impl Spell {
    pub fn make_instance(&self) -> SpellInstance<'_> {
        SpellInstance {
            charged_time: 0.,
            cooldown_time_left: 0.,
            ratio: 1.0,
            being_charged: false,
            spell: self,
        }
    }
}
impl<'a> SpellInstance<'a>{
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()>{
        if self.charged_time+DELTA >= self.spell.charge_time {
            self.being_charged = false;
            self.charged_time = 0.0;
        } else if self.being_charged {
            self.charged_time += DELTA;
        } else {
            self.charged_time = 0.0;
        }
        if self.cooldown_time_left > DELTA {
            self.cooldown_time_left -= DELTA
        } else {
            self.cooldown_time_left = 0.;
        }
        Ok(())
    }

    pub fn cast(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<ObjMaker<'a>>> {
        // if self.being_charged {
            mplayer.play(ctx, "throw")?;
            match self.spell.cast_type {
                CastType::EndOfCharge => {
                    if (self.spell.charge_time - self.charged_time ) > DELTA {
                        return Ok(None)
                    }
                }
                CastType::ChargeUp{..}  => {
                    self.ratio = self.charged_time/self.spell.charge_time;
                }
                CastType::WhileCharging{frequency} => {
                    if (((self.charged_time+0.01)/self.spell.charge_time) % frequency) > 0.011 {
                        return Ok(None)
                    }
                }
            }
            match self.spell.spell_type {
                SpellType::Explosion => Ok(Some(ObjMaker::Explosion(ExplosionMaker(self.spell, &self.spell.pattern, self.ratio)))),
                SpellType::Projectile => Ok(Some(ObjMaker::Projectile(ProjectileMaker(self.spell, &self.spell.pattern, self.ratio)))),
            }
        // } else {
        //     Ok(None)
        // }
    }
}