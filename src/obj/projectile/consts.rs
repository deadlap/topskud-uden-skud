use super::{Projectile};
use crate::util::{sstr, add_sstr, Sstr};

use lazy_static::lazy_static;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

lazy_static!{
    pub static ref PROJECTILES: HashMap<&'static str, Projectile> = {
        let mut file = File::open("resources/spells/projectiles/projectile_specs.toml").expect("specs.toml file");
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).expect("Reading to succeed");
        
        let templates: HashMap<Box<str>, ProjectileTemplate> = toml::from_str(&file_contents).expect("well-defined projectiles");
        templates.into_iter().map(|(k, v)| {
            let k = sstr(k);
            (k, v.build(k))
        }).collect()
    };
}

#[inline]
fn def_impact() -> Sstr {
    add_sstr("impact")
}
#[inline]
const fn def_speed() -> f32 {
    1200.
}

#[derive(Serialize, Deserialize)]
pub struct ProjectileTemplate {
    damage: f32,
    penetration: f32,
    #[serde(default = "def_impact")]
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    impact_snd: Sstr,
    #[serde(deserialize_with = "crate::util::deserialize_sstr")]
    entity_sprite: Sstr,
    #[serde(default = "def_speed")]
    speed: f32,
    range: f32,
}

impl ProjectileTemplate {
    fn build(self, id: &'static str) -> Projectile {
        let ProjectileTemplate {
            damage,
            penetration,
            impact_snd,
            entity_sprite,
            speed,
            range,
        } = self;

        Projectile {
            id,
            damage,
            penetration,
            impact_snd,
            entity_sprite,
            speed,
            range,
        }
    }
}