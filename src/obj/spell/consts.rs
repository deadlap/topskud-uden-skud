use super::{Element, SpellType, CastType, Spell};
use crate::util::{sstr, add_sstr, Sstr};

use lazy_static::lazy_static;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::f32::consts::PI;

lazy_static!{
    pub static ref SPELLS: HashMap<&'static str, Spell> = {
        let mut file = File::open("resources/spells/specs.toml").expect("specs.toml file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Reading to succeed");
        
        let templates: HashMap<Box<str>, SpellTemplate> = toml::from_str(&file_contents).expect("well-defined spells");
        templates.into_iter().map(|(k, v)| {
            let k = sstr(k);
            (k, v.build(k))
        }).collect()
    };
}

#[derive(Serialize, Deserialize)]
pub struct SpellTemplate {
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    name: Sstr,
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    cast_name: Sstr,
    element_type: Vec<Element>,
    energy_cost: f32,
    spell_type: SpellType,
    #[serde(default)]
    cast_type: CastType,
    #[serde(default = "def_range")]
    spell_range: f32,
    cooldown_time: f32,
    #[serde(default = "def_charge_time")]
    charge_time: f32,
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    cast_snd: Sstr,
    #[serde(default = "def_charge")]
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    charge_snd: Sstr,
    pattern: Vec<f32>,
}

#[inline]
const fn def_charge_time() -> f32 {
    0.25
}
#[inline]
fn def_charge() -> Sstr {
    add_sstr("ding")
}
#[inline]
const fn def_range() -> f32 {
    144.
}
const DEG2RAD: f32 = PI / 180.;

impl SpellTemplate {
    fn build(self, id: &'static str) -> Spell {
        let SpellTemplate {
            name,
            cast_name,
            element_type,
            energy_cost,
            spell_type,
            cast_type,
            spell_range,
            cooldown_time,
            charge_time,
            cast_snd,
            charge_snd,
            pattern,
        } = self;

        Spell {
            id,
            name,
            cast_name,
            element_type,
            energy_cost,
            spell_type,
            cast_type,
            spell_range,
            cooldown_time,
            charge_time,
            cast_snd,
            charge_snd,
            pattern: pattern.into_iter().map(|deg| deg * DEG2RAD).collect(),
            // hands_sprite: sstr(entity_sprite.to_string() + "_hands"),
        }
    }
}