use std::f32::consts::FRAC_1_SQRT_2 as COS_45_D;
use crate::{
    ext::FloatExt,
    util::{
        BLUE, GREEN, RED,
        angle_to_vec, angle_from_vec,
        ver, hor,
        Vector2, Point2
    },
    io::tex::PosText,
    obj::{Object, decal::Decal, pickup::Pickup, player::{Player}, enemy::{Enemy, Chaser}, health::Health},
    game::{
        DELTA, State, GameState, StateSwitch, world::{Level, Statistics, World},
        event::{Event::{self, Key, Mouse}, MouseButton, KeyCode, KeyMods}
    },
};
use ggez::{
    Context, GameResult,
    graphics::{
        self, Drawable, DrawMode, Rect,
        Color, DrawParam,
        MeshBuilder, Mesh, WHITE,
        spritebatch::SpriteBatch,
    },
    input::{
        keyboard,
        mouse,
    },
};

use rand::{thread_rng, prelude::SliceRandom};

pub fn new_blood(mut obj: Object) -> Decal {
    obj.pos += 16. * angle_to_vec(obj.rot);
    Decal {
        obj,
        spr: [
            "common/blood1",
            "common/blood2",
            "common/blood2",
            "common/blood3",
            "common/blood3",
        ].choose(&mut thread_rng()).copied().map(Into::into).unwrap(),
    }
}

/// The state of the game
pub struct Play {
    hp_text: PosText,
    arm_text: PosText,
    status_text: PosText,
    hud: Hud,
    world: World,
    holes: SpriteBatch,
    cur_pickup: Option<usize>,
    victory_time: f32,
    time: usize,
    initial: Health,
    level: Level,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State, level: Level, pl: Option<(Health)>) -> GameResult<Box<dyn GameState>> {
        mouse::set_cursor_hidden(ctx, true);

        let mut player = Player::from_point(level.start_point.unwrap_or_else(|| Point2::new(500., 500.)));
        if let Some(h) = pl {
            player = player.with_health(h);
        };

        Ok(Box::new(
            Play {
                level: level.clone(),
                initial: (player.health),
                hp_text: s.assets.text(Point2::new(4., 4.)).and_text("100"),
                arm_text: s.assets.text(Point2::new(4., 33.)).and_text("100"),
                status_text: s.assets.text(Point2::new(s.width as f32 / 2., s.height as f32 / 2. + 32.)).and_text(""),
                hud: Hud::new(ctx)?,
                time: 0,
                victory_time: 0.,
                cur_pickup: None,
                world: {
                    let mut world = World {
                        enemies: level.enemies,
                        player,
                        palette: level.palette,
                        grid: level.grid,
                        exit: level.exit,
                        intels: level.intels,
                        decals: level.decals,
                        pickups: level.pickups.into_iter().map(|(p, i)| Pickup::new(p, i)).collect(),
                    };
                    world.enemy_pickup();
                    world.player_pickup();

                    world
                },
                holes: SpriteBatch::new(s.assets.get_img(ctx, "common/hole").clone()),
            }
        ))
    }
}

impl GameState for Play {
    #[allow(clippy::cognitive_complexity)]
    fn update(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        self.hp_text.update(0, format!("{:02.0}", self.world.player.health.hp))?;
        self.arm_text.update(0, format!("{:02.0}", self.world.player.health.armour))?;
        
        self.status_text.update(0, "")?;

        let mut deads = Vec::new();
        for (i, &intel) in self.world.intels.iter().enumerate().rev() {
            if (intel-self.world.player.obj.pos).norm() <= 15. {
                deads.push(i);
                s.mplayer.play(ctx, "hit")?;
            }
        }
        for i in deads {
            self.world.intels.remove(i);
        }
        let mut deads = Vec::new();
        for (i, pickup) in self.world.pickups.iter().enumerate().rev() {
            if (pickup.pos-self.world.player.obj.pos).norm() <= 15. && pickup.apply(&mut self.world.player.health) {
                deads.push(i);
                s.mplayer.play(ctx, "hit")?;
            }
        }
        for i in deads {
            self.world.pickups.remove(i);
        }
        self.cur_pickup = None;

        // Define player velocity here already because enemies need it
        let player_vel = Vector2::new(hor(&ctx), ver(&ctx));

        for enemy in self.world.enemies.iter_mut() {
            if enemy.can_see(self.world.player.obj.pos, &self.world.palette, &self.world.grid) {
                enemy.behaviour = Chaser::LastKnown{
                    pos: self.world.player.obj.pos,
                    vel: player_vel,
                };
            }
            enemy.update(ctx, &mut s.mplayer)?;
        }

        let speed = if !keyboard::is_mod_active(ctx, KeyMods::SHIFT) {
            200.
        } else {
            100.
        };
        self.world.player.obj.move_on_grid(player_vel, speed, &self.world.palette, &self.world.grid);

        let game_won = match self.world.exit {
            Some(p) => self.world.intels.is_empty() && (p - self.world.player.obj.pos).norm() < 32.,
            None => self.world.enemies.is_empty(),
        };

        if game_won && self.victory_time <= 0. {
            s.mplayer.play(ctx, "victory")?;
            self.victory_time += DELTA;
        } else if self.victory_time > 0. {
            self.victory_time += DELTA;
        } else {
            self.time += 1;
        }
        if self.victory_time >= 2. {
            s.switch(StateSwitch::Win(Box::new(Statistics{
                level: self.level.clone(),
                time: self.time,
                enemies_left: self.world.enemies.len(),
                health_left: self.world.player.health,
            })));
        }
        Ok(())
    }
    fn logic(&mut self, s: &mut State, ctx: &mut Context) -> GameResult<()> {
        let dist = s.mouse - s.offset - self.world.player.obj.pos;

        self.hud.update_bars(ctx, &self.world.player)?;

        self.world.player.obj.rot = angle_from_vec(dist);

        // Center the camera on the player
        let p = self.world.player.obj.pos;
        s.focus_on(p);
        Ok(())
    }

    fn draw(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.world.grid.draw(&self.world.palette, ctx, &s.assets)?;

        self.holes.draw(ctx, Default::default())?;

        for &intel in &self.world.intels {
            let drawparams = graphics::DrawParam {
                dest: intel.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, "common/intel");
            graphics::draw(ctx, &*img, drawparams)?;
        }
        for decal in &self.world.decals {
            decal.draw(ctx, &s.assets, WHITE)?;
        }

        for pickup in &self.world.pickups {
            let drawparams = graphics::DrawParam {
                dest: pickup.pos.into(),
                offset: Point2::new(0.5, 0.5).into(),
                .. Default::default()
            };
            let img = s.assets.get_img(ctx, pickup.pickup_type.spr);
            graphics::draw(ctx, &*img, drawparams)?;
        }

        self.world.player.draw_player(ctx, &s.assets)?;

        for enemy in &self.world.enemies {
            enemy.draw(ctx, &s.assets, WHITE)?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.hud.draw(ctx)?;

        self.hp_text.draw_text(ctx)?;
        self.arm_text.draw_text(ctx)?;
        self.status_text.draw_center(ctx)?;
        
        let drawparams = graphics::DrawParam {
            dest: s.mouse.into(),
            offset: Point2::new(0.5, 0.5).into(),
            color: RED,
            .. Default::default()
        };
        let img = s.assets.get_img(ctx, "common/crosshair");
        graphics::draw(ctx, &*img, drawparams)
    }
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        use self::KeyCode::*;
        match event {
            Key(G) => {
                warn!("Dropped nothing");
            },
            Key(R) => {},
            Key(F) => {},
            Mouse(MouseButton::Left) | Key(Space) => {
                // TODO do knives with bullets too
                let player = &mut self.world.player;
                let mut backstab = false;
                let mut dead = None;

                for (i, enemy) in self.world.enemies.iter_mut().enumerate() {
                    let dist = player.obj.pos-enemy.pl.obj.pos;
                    let dist_len = dist.norm();
                    if dist_len < 44. {
                        backstab = angle_to_vec(enemy.pl.obj.rot).dot(&dist) / dist_len < COS_45_D;

                        self.world.decals.push(new_blood(enemy.pl.obj.clone()));
                        enemy.pl.health.weapon_damage(if backstab { 165. } else { 33. }, 0.92);
                        if enemy.pl.health.is_dead() {
                            dead = Some(i);
                            break;
                        }
                    }
                }
                if let Some(i) = dead {
                    s.mplayer.play(ctx, "death").unwrap();

                    let Enemy{pl: Player{ obj: Object{pos, ..}, ..}, ..}
                        = self.world.enemies.remove(i);
                }

                s.mplayer.play(ctx, if backstab {"shuk"} else {"hling"}).unwrap();
            }
            _ => (),
        }
    }

    fn get_world(&self) -> Option<&World> {
        Some(&self.world)
    }
    fn get_mut_world(&mut self) -> Option<&mut World> {
        Some(&mut self.world)
    }
}

#[derive(Debug)]
pub struct Hud {
    hud_bar: Mesh,
    hp_bar: Mesh,
    armour_bar: Mesh,
}

const RECTS: [Rect; 4] = [
    Rect{x:104.,y:2.,h: 32., w: 32.},
    Rect{x:137.,y:2.,h: 32., w: 32.},
    Rect{x:104.,y:35.,h: 32., w: 32.},
    Rect{x:137.,y:35.,h: 32., w: 32.}
];

impl Hud {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let hud_bar = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 1., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 29., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 57., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x:104.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .rectangle(DrawMode::fill(), Rect{x:137.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .rectangle(DrawMode::fill(), Rect{x:104.,y:35.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .rectangle(DrawMode::fill(), Rect{x:137.,y:35.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .build(ctx)?;

        let hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: 0., h: 24.}, GREEN)?;
        let armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: 0., h: 24.}, BLUE)?;

        Ok(Hud{
            hud_bar,
            hp_bar,
            armour_bar,
        })
    }
    pub fn update_bars(&mut self, ctx: &mut Context, p: &Player) -> GameResult<()> {
        self.hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: p.health.hp.limit(0., 100.), h: 24.}, GREEN)?;
        self.armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: p.health.armour.limit(0., 100.), h: 24.}, BLUE)?;

        Ok(())
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.hud_bar.draw(ctx, Default::default())?;
        self.hp_bar.draw(ctx, Default::default())?;
        self.armour_bar.draw(ctx, Default::default())
    }
}