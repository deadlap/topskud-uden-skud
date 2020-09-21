use super::{Explosion};
use crate::util::{sstr, Sstr};

use lazy_static::lazy_static;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use std::f32::consts::PI;

const DEG2RAD: f32 = PI / 180.;

lazy_static!{
    pub static ref EXPLOSIONS: HashMap<&'static str, Explosion> = {
        let mut file = File::open("resources/spells/explosions/explosion_specs.toml").expect("specs.toml file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Reading to succeed");
        
        let templates: HashMap<Box<str>, ExplosionTemplate> = toml::from_str(&file_contents).expect("well-defined explosions");
        templates.into_iter().map(|(k, v)| {
            let k = sstr(k);
            (k, v.build(k))
        }).collect()
    };
}

#[inline]
const fn def_degrees() -> f32 {
    360.
}

#[derive(Serialize, Deserialize)]
pub struct ExplosionTemplate {
    low_damage: f32,
    high_damage: f32,
    penetration: f32,
    start_fuse: f32,
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    entity_sprite: Sstr,
    range: f32,
    #[serde(default = "def_degrees")]
    degrees: f32,
    lethal_range: f32,
}

impl ExplosionTemplate {
    fn build(self, id: &'static str) -> Explosion {
        let ExplosionTemplate {
            low_damage,
            high_damage,
            penetration,
            start_fuse,
            entity_sprite,
            range,
            degrees,
            lethal_range,
        } = self;

        Explosion {
            id,
            low_damage,
            high_damage,
            penetration,
            start_fuse,
            entity_sprite,
            range,
            degrees: degrees*DEG2RAD,
            lethal_range,
        }
    }
}