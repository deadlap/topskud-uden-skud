use ggez::{Context, GameResult, graphics::{self, WHITE, Color, Mesh, DrawParam}};
use std::{iter, f32::consts::{PI, FRAC_PI_2 as HALF_PI}};
use rand::{thread_rng, Rng};

const PI_MUL_2: f32 = 2. * PI;

use crate::{
    util::{angle_to_vec, Vector2},
    game::{
        DELTA,
        world::{Grid, Palette},
    },
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
};
use super::{Object, player::Player, enemy::Enemy, health::Health};

#[derive(Debug, Default, Clone, Copy)]
pub struct Utilities {
    pub explosions: u8,
}

#[derive(Debug, Clone)]
pub struct Explosion {
    pub obj: Object,
    pub state: ExplosionState,
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
const DEC: f32 = 1.4;

const RANGE: f32 = 144.;
const LETHAL_RANGE: f32 = 64.;

impl Explosion {
    #[inline]
    pub fn apply_damage(health: &mut Health, high: bool) {
        health.weapon_damage(if high { 105.} else {55.}, 0.85);
    }
    #[inline]
    pub fn draw(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        match &self.state {
            ExplosionState::Fused{..} => {
                // let img = a.get_img(ctx, "weapons/pineapple");
                // self.obj.draw(ctx, &*img, WHITE)
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
        const RADIANS_PER_VERT: f32 = (360. / NUM_VERTICES as f32) * PI/180.;

        let random_offset = thread_rng().gen_range(0., PI_MUL_2);

        let centre = graphics::Vertex {
            pos: [0., 0.],
            uv: [0.5, 0.5],
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let vertices: Vec<_> = (0..NUM_VERTICES).map(|i| {
            let angle = RANGE * angle_to_vec(i as f32 * RADIANS_PER_VERT);
            let angle_uv = 0.5 * angle_to_vec(i as f32 * RADIANS_PER_VERT + random_offset);
            let cast = grid.ray_cast(palette, self.obj.pos, angle, true);
            graphics::Vertex{
                pos: (cast.into_point() - self.obj.pos).into(),
                uv: (Vector2::new(0.5, 0.5) + (cast.clip().norm()-RANGE)/RANGE * angle_uv).into(),
                color: [1.0, 1.0, 1.0, 1.0],
            }
        }).chain(iter::once(centre)).collect();
        
        let indices = (0..NUM_VERTICES).flat_map(|i| iter::once(NUM_VERTICES).chain(iter::once(i)).chain(iter::once((i + 1) % NUM_VERTICES))).collect::<Vec<_>>();
        let expl_img = (a.get_img(ctx, "weapons/explosion1")).clone();
        Mesh::from_raw(ctx, &vertices, &indices, Some(expl_img))
    }
    pub fn update_fused(obj: &mut Object, fuse: &mut f32, palette: &Palette, grid: &Grid, player: &mut Player, enemies: &mut [Enemy]) -> ExplosionUpdate {
        let start = obj.pos;
        if *fuse > DELTA {
            *fuse -= DELTA;
        } else {
            *fuse = 0.;

            let player_hit;
            let mut enemy_hits = Vec::new();

            let d_player = player.obj.pos-start;
            if d_player.norm() < RANGE && grid.ray_cast(palette, start, d_player, true).full() {
                Self::apply_damage(&mut player.health, d_player.norm() <= LETHAL_RANGE);
                player_hit = true;
            } else {
                player_hit = false;
            }

            for (i, enem) in enemies.iter_mut().enumerate().rev() {
                let d_enemy = enem.pl.obj.pos - start;
                if d_enemy.norm() < 144. && grid.ray_cast(palette, start, d_enemy, true).full() {
                    Self::apply_damage(&mut enem.pl.health, d_enemy.norm() <= 64.);
                    enemy_hits.push(i);
                }
            }

            return ExplosionUpdate::Explosion{player_hit, enemy_hits};
        }

        let closest_p = Grid::closest_point_of_line_to_circle(start, d_pos, player.obj.pos);
        let r_player = player.obj.pos - closest_p;
        
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
            ExplosionState::Fused{ref mut fuse} => {
                Self::update_fused(&mut self.obj, fuse, palette, grid, player, enemies)
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

impl Utilities {
    pub fn Create_Explosion(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<Option<ExplosionMaker>> {
        if self.explosions > 0 {
            self.explosions -= 1;

            mplayer.play(ctx, "throw")?;
            Ok(Some(ExplosionMaker(620.)))
        } else {
            mplayer.play(ctx, "cock")?;
            Ok(None)
        }
    }
}

pub struct ExplosionMaker(f32);
impl ExplosionMaker {
    pub fn make(self, mut obj: Object) -> Explosion {
        obj.rot = 0.;
        Explosion {
            state: ExplosionState::Fused{fuse: 0.05},
            obj,
        }
    }
    pub fn make_with_fuse(self, mut obj: Object, fuse: f32) -> Explosion {
        obj.rot = 0.;
        Explosion {
            state: ExplosionState::Fused{fuse},
            obj,
        }
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