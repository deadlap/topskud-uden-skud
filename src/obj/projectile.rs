use ggez::{Context, GameResult, graphics::WHITE};

use crate::{
    util::{Vector2, Sstr},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::tex::{Assets, }
};
use super::{Object, player::Player, enemy::Enemy, health::Health, spell::{Spell}};

#[derive(Debug, Clone)]
pub struct Projectile<> {
    pub id: Sstr,
    pub damage: f32,
    pub penetration: f32,
    pub impact_snd: Sstr,
    pub entity_sprite: Sstr,
    pub speed: f32,
    pub range: f32,
}

#[derive(Debug, Clone)]
pub struct ProjectileInstance<'a> {
    pub obj: Object,
    pub vel: Vector2,
    pub spell: &'a Spell,
    pub projectile: &'a Projectile,
}

mod consts;
pub use self::consts::*;

impl ProjectileInstance<'_> {
    pub fn apply_damage(&self, health: &mut Health) {
        let dmg = self.projectile.damage * self.vel.norm() / self.projectile.speed;

        health.weapon_damage(dmg, self.projectile.penetration);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        // let img = a.get_img(ctx, "common/bullet");
        let img = a.get_img(ctx, self.projectile.entity_sprite);
        self.obj.draw(ctx, &*img, WHITE)
    }
    pub fn update(&mut self, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> Hit {
        let start = self.obj.pos;
        let d_pos = self.vel * DELTA;

        const VELOCITY_DECREASE: f32 = 220. * DELTA;

        if self.vel.norm() <= VELOCITY_DECREASE {
            return Hit::Wall
        }
        
        // Check if we've hit a player or an enemy
        if Grid::dist_line_circle(start, d_pos, player.obj.pos) <= 16. {
            self.apply_damage(&mut player.health);
            return Hit::Player;
        }
        for (i, enem) in enemies.iter_mut().enumerate() {
            if Grid::dist_line_circle(start, d_pos, enem.pl.obj.pos) <= 16. {
                self.apply_damage(&mut enem.pl.health);
                return Hit::Enemy(i);
            }
        }

        // Decrease velocity after damage could've been dealt
        self.vel -= self.vel.normalize() * VELOCITY_DECREASE;

        // Ray cast projectile to see if we've hit a wall and move projectile accordingly
        let cast = grid.ray_cast(palette, start, d_pos, true);
        self.obj.pos = cast.into_point();
        if cast.full() {
            Hit::None
        } else {
            Hit::Wall
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Hit {
    Wall,
    Player,
    Enemy(usize),
    None,
}