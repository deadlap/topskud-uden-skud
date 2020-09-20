use ggez::{Context, GameResult, graphics::{self, WHITE, Color, Mesh, DrawParam}};
use std::{iter, f32::consts::{PI, FRAC_PI_2 as HALF_PI}};
use rand::{thread_rng, Rng};
const PI_MUL_2: f32 = 2. * PI;

use crate::{
    util::{angle_to_vec, Vector2, Sstr, Point2},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
};
use super::{Object, player::Player, enemy::Enemy, health::Health, spell::{Spell}};

#[derive(Debug, Clone)]
pub struct Explosion {
    pub id: Sstr,
    pub start_fuse: f32,
    pub range: f32,
    pub degrees: f32,
    pub low_damage: f32,
    pub high_damage: f32,
    pub penetration: f32,
    pub entity_sprite: Sstr,
    pub lethal_range: f32,
}

mod consts;
pub use self::consts::*;

#[derive(Debug, Clone)]
pub struct ExplosionInstance<'a> {
    pub updated_degrees: f32,
    pub obj: Object,
    pub fuse: f32,
    pub state: ExplosionState,
    pub spell: &'a Spell,
    pub explosion: &'a Explosion,
}

#[derive(Debug, Clone)]
pub enum ExplosionState {
    Fused {
        fuse: f32,
    },
    Explosion {
        alive_time: f32,
        mesh: Mesh,
    }
}

const EXPLOSION_LIFETIME: f32 = 0.5;

impl ExplosionInstance<'_> {
    #[inline]
    pub fn apply_damage(&self, health: &mut Health, high: bool) {
        health.weapon_damage(if high {self.explosion.high_damage} else {self.explosion.low_damage}, self.explosion.penetration);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        match &self.state {
            ExplosionState::Fused{..} => {
                let img = a.get_img(ctx, "spells/explosions/smoke");
                self.obj.draw(ctx, &*img, WHITE)
            }
            ExplosionState::Explosion { mesh, alive_time } => {
                const EXPANDING_TIME: f32 = 0.1;
                let mut dp = DrawParam::from((self.obj.pos,));

                if *alive_time <= EXPANDING_TIME {
                    let scale = alive_time / EXPANDING_TIME;
                    dp = dp.scale(Vector2::new(scale, scale));
                } else {
                    let colour = (HALF_PI * (alive_time - EXPANDING_TIME) / (EXPLOSION_LIFETIME - EXPANDING_TIME)).cos();
                    dp = dp.color(Color{r: colour, g: colour, b: colour, a: 0.5+0.5*colour});
                }

                graphics::draw(ctx, mesh, dp)
            }
        }
    }
    fn make_mesh(&self, ctx: &mut Context, a: &Assets, palette: &Palette, grid: &Grid) -> GameResult<Mesh> {
        const NUM_VERTICES: u32 = 120;
        let radians_per_vert: f32 = (self.updated_degrees / NUM_VERTICES as f32) * PI/180.;
        let angle_offset = self.obj.rot+(self.updated_degrees/2.)*PI/180.;
        
        let random_offset = thread_rng().gen_range(0., PI_MUL_2);

        let centre = graphics::Vertex {
            pos: [0., 0.],
            uv: [0.5, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let vertices: Vec<_> = (0..NUM_VERTICES).map(|i| {
            let angle = self.explosion.range * angle_to_vec(angle_offset - i as f32 * radians_per_vert);
            let angle_uv = 0.5 * angle_to_vec(i as f32 * radians_per_vert + random_offset);
            let cast = grid.ray_cast(palette, self.obj.pos, angle, true);
            graphics::Vertex{
                pos: (cast.into_point() - self.obj.pos).into(),
                uv: (Vector2::new(0.5, 0.5) + (cast.clip().norm()-self.explosion.range)/self.explosion.range * angle_uv).into(),
                color: [1.0, 1.0, 1.0, 1.0],
            }
        }).chain(iter::once(centre)).collect();
        
        let indices = (0..NUM_VERTICES).flat_map(|i| iter::once(NUM_VERTICES).chain(iter::once(i)).chain(iter::once((i + 1) % NUM_VERTICES))).collect::<Vec<_>>();
        let expl_img = (a.get_img(ctx, self.explosion.entity_sprite)).clone();
        Mesh::from_raw(ctx, &vertices, &indices, Some(expl_img))
    }
    fn is_pos_hit(&self, palette: &Palette, grid: &Grid, obj: &Object) -> bool {
        let start = self.obj.pos;
        let d_pos = obj.pos-start;
        let rot_offset = (self.updated_degrees/2.)*PI/180.;
        let vec_start = angle_to_vec(self.obj.rot+rot_offset);
        let vec_end = angle_to_vec(self.obj.rot-rot_offset);
        let in_range = d_pos.norm() < self.explosion.range && grid.ray_cast(palette, start, d_pos, true).full();
        (in_range && 
            !(-vec_start[0]*d_pos[1]+vec_start[1]*d_pos[0] > 0.) &&
            (-vec_end[0]*d_pos[1]+vec_end[1]*d_pos[0] > 0.))
    }
    pub fn update_fused(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> ExplosionUpdate {
        let start = self.obj.pos;
        if self.fuse > DELTA {
            self.fuse -= DELTA;
        } else {
            self.fuse = 0.;

            let player_hit;
            let mut enemy_hits = Vec::new();
            let d_player = player.obj.pos-start;
            if self.is_pos_hit(palette, grid, &player.obj) {
                Self::apply_damage(&self, &mut player.health, d_player.norm() <= self.explosion.lethal_range);
                player_hit = true;
            } else {
                player_hit = false;
            }

            for (i, enem) in enemies.iter_mut().enumerate().rev() {
                let d_enemy = enem.pl.obj.pos - start;
                if self.is_pos_hit(palette, grid, &enem.pl.obj) {
                    Self::apply_damage(self, &mut enem.pl.health, d_enemy.norm() <= 64.);
                    enemy_hits.push(i);
                }
            }

            return ExplosionUpdate::Explosion{player_hit, enemy_hits};
        }
        
        ExplosionUpdate::None
    }

    pub fn update(&mut self, ctx: &mut Context, a: &Assets, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> GameResult<ExplosionUpdate> {
        let update = match self.state {
            ExplosionState::Explosion{ref mut alive_time, ..} => {
                *alive_time += DELTA;
                if *alive_time >= EXPLOSION_LIFETIME {
                    ExplosionUpdate::Dead
                } else {
                    ExplosionUpdate::None
                }
            }
            ExplosionState::Fused{..} => {
                Self::update_fused(self, palette, grid, player, enemies)
            }
        };
        if let ExplosionUpdate::Explosion{..} = update {
            self.state = ExplosionState::Explosion {
                alive_time: 0.,
                mesh: self.make_mesh(ctx, a, palette, grid)?
            };
        }
        Ok(update)
    }
}

pub struct ExplosionMaker<'a>(pub &'a Spell, pub &'a Vec<f32>);
impl<'a> ExplosionMaker<'a> {
    pub fn make(self, obj: Object) -> impl Iterator<Item=ExplosionInstance<'a>> {
        let ExplosionMaker(spell, offsets) = self;
        let explosion = &EXPLOSIONS[spell.cast_name];

        offsets.into_iter().map(move |offset| {
            let mut obj = obj.clone();

            obj.rot += offset;
            obj.pos += spell.spell_range * angle_to_vec(obj.rot);
            ExplosionInstance {
                updated_degrees: if obj.rot > 0. {1.} else {-1.} * explosion.degrees,
                obj,
                spell,
                fuse: explosion.start_fuse,
                state: ExplosionState::Fused{fuse: explosion.start_fuse},
                explosion,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum ExplosionUpdate {
    Explosion {
        player_hit: bool,
        enemy_hits: Vec<usize>,
    },
    Dead,
    None,
}