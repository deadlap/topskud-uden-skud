use std::f32::consts::{FRAC_1_SQRT_2 as COS_45_D};
use crate::{
    ext::FloatExt,
    util::{
        BLUE, GREEN, RED,
        angle_to_vec, angle_from_vec,
        ver, hor,
        Vector2, Point2
    },
    io::tex::PosText,
    obj::{Object, explosion::{ExplosionUpdate}, 
        decal::Decal, pickup::Pickup, player::{Player, ActiveSlot, ElemSlots}, energy::Energy, 
        enemy::{Enemy, Chaser}, health::Health, spell::ObjMaker},
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
    energy_text: PosText,
    // cooldown_text: PosText,
    status_text: PosText,
    hud: Hud,
    world: World,
    holes: SpriteBatch,
    cur_pickup: Option<usize>,
    victory_time: f32,
    time: usize,
    initial: (Health, Energy, ElemSlots),
    level: Level,
}

impl Play {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(ctx: &mut Context, s: &mut State, level: Level, pl: Option<(Health, Energy, ElemSlots)>) -> GameResult<Box<dyn GameState>> {
        mouse::set_cursor_hidden(ctx, true);

        let mut player = Player::from_point(level.start_point.unwrap_or_else(|| Point2::new(500., 500.)));
        if let Some((h, e, es)) = pl {
            player = player.with_health(h).with_energy(e).with_spell(es);
        };

        Ok(Box::new(
            Play {
                level: level.clone(),
                initial: (player.health, player.energy, player.spell.clone()),
                hp_text: s.assets.text(Point2::new(4., 4.)).and_text("100"),
                arm_text: s.assets.text(Point2::new(4., 33.)).and_text("100"),
                energy_text: s.assets.text(Point2::new(4., 62.)).and_text("100.0"),
                // cooldown_text: s.assets.text(Point2::new(2., 87.)).and_text("0.0").and_text("s"),
                status_text: s.assets.text(Point2::new(s.width as f32 / 2., s.height as f32 / 2. + 32.)).and_text(""),
                hud: Hud::new(ctx)?,
                time: 0,
                victory_time: 0.,
                cur_pickup: None,
                world: {
                    let mut world = World {
                        enemies: level.enemies,
                        player,
                        explosions: Vec::new(),
                        projectiles: Vec::new(),
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
        self.world.player.update(ctx, &mut s.mplayer)?;
        self.hp_text.update(0, format!("{:02.0}", self.world.player.health.hp))?;
        self.arm_text.update(0, format!("{:02.0}", self.world.player.health.armour))?;
        // self.energy_text.update(0, format!("{:.1}", self.world.player.energy.cur_energy))?;
        self.energy_text.update(0, format!("{:02.0}", self.world.player.energy.cur_energy))?;
        // if let Some(spell) = self.world.player.spell.get_active() {
        //     self.cooldown_text.update(0, format!("{:02.0}", spell.cooldown_time_left))?;
        // }
        self.status_text.update(0, "")?;

        let mut deads = Vec::new();
        for (i, explosion) in self.world.explosions.iter_mut().enumerate().rev() {
            let e_update = explosion.update(ctx, &s.assets, &self.world.palette, &self.world.grid, &mut self.world.player, &mut *self.world.enemies)?;

            match e_update {
                ExplosionUpdate::Explosion{player_hit, enemy_hits} => {
                    s.mplayer.play(ctx, "boom")?;

                    if player_hit {
                        self.world.decals.push(new_blood(self.world.player.obj.clone()));
                        s.mplayer.play(ctx, "hit")?;

                        if self.world.player.health.is_dead() {
                            s.switch(StateSwitch::Lose(Box::new(Statistics{
                                time: self.time,
                                enemies_left: self.world.enemies.len(),
                                health_left: self.initial.0,
                                energy_left: self.initial.1,
                                spell: self.initial.2.clone(),
                                level: self.level.clone(),
                            })));
                            s.mplayer.play(ctx, "death")?;
                        } else {
                            s.mplayer.play(ctx, "hurt")?;
                        }
                    }
                    for i in enemy_hits {
                        let enemy = &self.world.enemies[i];
                        s.mplayer.play(ctx, "hit")?;

                        self.world.decals.push(new_blood(enemy.pl.obj.clone()));
                        if enemy.pl.health.is_dead() {
                            s.mplayer.play(ctx, "death")?;

                            let Enemy{pl: Player{obj: Object{ ..}, ..}, ..}
                                = self.world.enemies.remove(i);
                        } else {
                            if !enemy.behaviour.chasing() {
                                self.world.enemies[i].behaviour = Chaser::LookAround{
                                    dir: explosion.obj.pos - enemy.pl.obj.pos
                                };
                            }
                            s.mplayer.play(ctx, "hurt")?;
                        }
                    }
                }
                ExplosionUpdate::Dead => {
                    deads.push(i);
                }
                ExplosionUpdate::None => (),
            }
        }
        for i in deads {
            self.world.explosions.remove(i);
        }

        let mut deads = Vec::new();
        for (i, projectile) in self.world.projectiles.iter_mut().enumerate().rev() {
            let hit = projectile.update(&self.world.palette, &self.world.grid, &mut self.world.player, &mut *self.world.enemies);

            use crate::obj::projectile::Hit;

            match hit {
                Hit::None => (),
                // _ if projectile.weapon.bullet_type == BulletType::Rocket => {
                //     let gren = Object::new(bullet.obj.pos - angle_to_vec(bullet.obj.rot));
                //     let nade = GrenadeMaker(0.).make_with_fuse(gren,0.);
                //     self.world.grenades.push(nade);
                //     // Du skal huske at slette bulletten bagefter, et òg @deadlap
                //     deads.push(i);
                // }
                Hit::Wall => {
                    s.mplayer.play(ctx, &projectile.projectile.impact_snd)?;
                    let dir = angle_to_vec(projectile.obj.rot);
                    projectile.obj.pos += Vector2::new(5.*dir.x.signum(), 5.*dir.y.signum());
                    self.holes.add(projectile.obj.drawparams());
                    deads.push(i);
                }
                Hit::Player => {
                    deads.push(i);
                    self.world.decals.push(new_blood(projectile.obj.clone()));
                    s.mplayer.play(ctx, "hit")?;

                    if self.world.player.health.is_dead() {
                        s.switch(StateSwitch::Lose(Box::new(Statistics{
                            time: self.time,
                            enemies_left: self.world.enemies.len(),
                            health_left: self.initial.0,
                            energy_left: self.initial.1,
                            spell: self.initial.2.clone(),
                            level: self.level.clone(),
                        })));
                        s.mplayer.play(ctx, "death")?;
                    } else {
                        s.mplayer.play(ctx, "hurt")?;
                    }
                }
                Hit::Enemy(e) => {
                    // if !bullet.weapon.bullet_type.piercing() {
                    // }
                    deads.push(i);
                    let enemy = &self.world.enemies[e];
                    s.mplayer.play(ctx, "hit")?;
                    
                    self.world.decals.push(new_blood(projectile.obj.clone()));
                    if enemy.pl.health.is_dead() {
                        s.mplayer.play(ctx, "death")?;
                        
                        let Enemy{pl: Player{obj: Object{..}, ..}, ..}
                        = self.world.enemies.remove(e);
                    } else {
                        if !enemy.behaviour.chasing() {
                            self.world.enemies[e].behaviour = Chaser::LookAround{
                                dir: projectile.obj.pos - enemy.pl.obj.pos
                            };
                        }
                        s.mplayer.play(ctx, "hurt")?;
                    }
                }
            }
        }
        for i in deads {
            self.world.projectiles.remove(i);
        }

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
        let player = &mut self.world.player;
        if let Some(cur_spell) = player.spell.get_cur_mut() {
            cur_spell.update(ctx, &mut s.mplayer)?;
            if mouse::button_pressed(ctx, MouseButton::Left) && !cur_spell.spell.cast_type.clone().is_charge_up() {
                cur_spell.being_charged = true;
                let pos = player.obj.pos;
                let mut obj = Object::new(pos);
                obj.rot = player.obj.rot;
                if let Some(obj_maker) = cur_spell.cast(ctx, &mut s.mplayer).unwrap() {
                    if player.energy.try_to_use_energy(cur_spell.spell.energy_cost) {
                        match obj_maker {
                            ObjMaker::Projectile(pm) => {
                                for projectile in pm.make(obj) {
                                    self.world.projectiles.push(projectile);
                                }
                            }
                            ObjMaker::Explosion(em) => {
                                for explosion in em.make(obj) {
                                    self.world.explosions.push(explosion);
                                }
                            }
                        }
                    }
                }
            }
        }
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
                energy_left: self.world.player.energy,
                spell: self.world.player.spell.clone(),
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
        for projectile in &self.world.projectiles {
            projectile.draw(ctx, &s.assets)?;
            // projectile.draw(ctx, &s.assets, &self.world.palette, &self.world.grid)?;
        }
        for explosion in &self.world.explosions {
            explosion.draw(ctx, &s.assets)?;
        }

        Ok(())
    }
    fn draw_hud(&mut self, s: &State, ctx: &mut Context) -> GameResult<()> {
        self.hud.draw(ctx)?;

        self.hp_text.draw_text(ctx)?;
        self.arm_text.draw_text(ctx)?;
        self.energy_text.draw_text(ctx)?;
        // self.cooldown_text.draw_text(ctx)?;
        self.status_text.draw_center(ctx)?;

        if let Some(slot_element) = &self.world.player.spell.slot {
            let drawparams = DrawParam::from(([104., 2.],));
            let img = s.assets.get_img(ctx, &slot_element.get_spr());
            graphics::draw(ctx, &*img, drawparams)?;
        }
        if let Some(slot_element) = &self.world.player.spell.slot2 {
            let drawparams = DrawParam::from(([137., 2.],));
            let img = s.assets.get_img(ctx, &slot_element.get_spr());
            graphics::draw(ctx, &*img, drawparams)?;
        }
        if let Some(slot_element) = &self.world.player.spell.slot3 {
            let drawparams = DrawParam::from(([170., 2.],));
            let img = s.assets.get_img(ctx, &slot_element.get_spr());
            graphics::draw(ctx, &*img, drawparams)?;
        }
        let selection = Mesh::new_rectangle(ctx, DrawMode::stroke(2.), RECTS[self.world.player.spell.active as u8 as usize], Color{r: 1., g: 1., b: 0., a: 1.})?;
        graphics::draw(ctx, &selection, DrawParam::default())?;

        let drawparams = graphics::DrawParam {
            dest: s.mouse.into(),
            offset: Point2::new(0.5, 0.5).into(),
            color: RED,
            .. Default::default()
        };
        let img = s.assets.get_img(ctx, "common/crosshair");
        graphics::draw(ctx, &*img, drawparams)
    }
    fn event_down(&mut self, _s: &mut State, _ctx: &mut Context, event: Event) {
        // use self::KeyCode::*;
        if let Mouse(MouseButton::Left) = event {
            let player = &mut self.world.player;
            if let Some(cur_spell) = player.spell.get_cur_mut() {
                if player.energy.try_to_use_energy(cur_spell.spell.energy_cost) {
                    cur_spell.being_charged = true;
                }
            }
        }
    }
    fn event_up(&mut self, s: &mut State, ctx: &mut Context, event: Event) {
        use self::KeyCode::*;
        match event {
            Key(Key1) | Key(Numpad1) => self.world.player.spell.switch(ActiveSlot::Slot),
            Key(Key2) | Key(Numpad2) => self.world.player.spell.switch(ActiveSlot::Slot2),
            Key(Key3) | Key(Numpad3) => self.world.player.spell.switch(ActiveSlot::Slot3),
            Key(G) => {
                warn!("Dropped nothing");
            },
            Mouse(MouseButton::Right) | Key(Space) => {
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

                    let Enemy{pl: Player{ obj: Object{..}, ..}, ..}
                        = self.world.enemies.remove(i);
                }

                s.mplayer.play(ctx, if backstab {"shuk"} else {"hling"}).unwrap();
            }
            Mouse(MouseButton::Left) => {
                let player = &mut self.world.player;
                if let Some(cur_spell) = player.spell.get_cur_mut() {
                    cur_spell.being_charged = false;
                    let pos = player.obj.pos;
                    let mut obj = Object::new(pos);
                    obj.rot = player.obj.rot;
                    if let Some(obj_maker) = cur_spell.cast(ctx, &mut s.mplayer).unwrap() {
                        if player.energy.try_to_use_energy(cur_spell.spell.energy_cost) {
                            match obj_maker {
                                ObjMaker::Projectile(pm) => {
                                    for projectile in pm.make(obj) {
                                        self.world.projectiles.push(projectile);
                                    }
                                }
                                ObjMaker::Explosion(em) => {
                                    for explosion in em.make(obj) {
                                        self.world.explosions.push(explosion);
                                    }
                                }
                            }
                        }
                    }
                }
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
    energy_bar: Mesh,
    // cooldown_bar: Mesh,
}

const RECTS: [Rect; 3] = [
    Rect{x:104.,y:2.,h: 32., w: 32.},
    Rect{x:137.,y:2.,h: 32., w: 32.},
    Rect{x:170.,y:2.,h: 32., w: 32.},
];

impl Hud {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let hud_bar = MeshBuilder::new()
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 1., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 29., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 57., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x: 1., y: 85., w: 102., h: 26.}, graphics::BLACK)
            .rectangle(DrawMode::fill(), Rect{x:104.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .rectangle(DrawMode::fill(), Rect{x:137.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .rectangle(DrawMode::fill(), Rect{x:170.,y:2.,h: 32., w: 32.}, Color{r: 0.5, g: 0.5, b: 0.5, a: 1.})
            .build(ctx)?;

        let hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: 0., h: 24.}, GREEN)?;
        let armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: 0., h: 24.}, BLUE)?;
        let energy_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: 0., h: 24.}, BLUE)?;
        // let cooldown_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 86., w: 0., h: 24.}, RED)?;

        Ok(Hud{
            hud_bar,
            hp_bar,
            armour_bar,
            energy_bar,
            // cooldown_bar,
        })
    }
    pub fn update_bars(&mut self, ctx: &mut Context, p: &Player) -> GameResult<()> {
        self.hp_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 2., w: p.health.hp.limit(0., 100.), h: 24.}, GREEN)?;
        self.armour_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 30., w: p.health.armour.limit(0., 100.), h: 24.}, BLUE)?;
        self.energy_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 58., w: p.energy.cur_energy.limit(0., p.energy.max_energy), h: 24.}, BLUE)?;
        // if let Some(spell) = p.spell.get_active() {
        //     self.cooldown_bar = Mesh::new_rectangle(ctx, DrawMode::fill(), Rect{x: 2., y: 86., 
        //         w: p.spell.get_active().map(|m| m.cooldown_time_left).unwrap_or(0.).limit(0., 1.)
        //             *100., h: 24.}, RED)?;
        // }
        Ok(())
    }
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        self.hud_bar.draw(ctx, Default::default())?;
        self.hp_bar.draw(ctx, Default::default())?;
        self.armour_bar.draw(ctx, Default::default())?;
        self.energy_bar.draw(ctx, Default::default())
        // self.cooldown_bar.draw(ctx, Default::default())
    }
}