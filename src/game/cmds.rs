use std::collections::HashMap;
use super::{Console, State, GameState, Command, CommandError, Content, StateSwitch};

use crate::{
    util::dbg_strs,
    obj::{health::Health, energy::Energy, spell::{Element}},
};
use ggez::Context;

macro_rules! commands {
    ($console:ident, $ctx:ident, $state:ident, $gs:ident, $args:ident, $(
        $($name:expr),+ => $f:block,
    )*) => {
        let mut map = HashMap::new();

        $({
            #[allow(unused_variables)]
            fn function<'a>($console: &'a mut Console, $ctx: &'a mut Context, $state: &'a mut State, $gs: &'a mut dyn GameState, $args: Vec<&'a str>) -> Result<(), CommandError> $f

            let cmd = function as Command;
            
            $(
                map.insert($name.into(), cmd);
            )+
        })*

        map
    };
}

pub(super) fn commands() -> HashMap<String, Command> {
    use CommandError::*;

    commands!{console, ctx, state, gs, args,
        "" => {Ok(())},
        "pi", "intels" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.intels.clear();
            info!("Intels got");
            Ok(())
        },
        "clear", "cl" => {
            console.history = state.assets.raw_text_with("", 18.);
            Ok(())
        },
        "fa", "fullarmour" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.player.health.hp = 100.;
            world.player.health.armour = 100.;
            Ok(())
        },
        "fe", "fullenergy" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            world.player.energy.cur_energy = 100.;
            Ok(())
        },
        "god" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            if world.player.health.hp.is_finite() {
                world.player.health.hp = std::f32::INFINITY;
                info!("Degreelessness");
            } else {
                world.player.health.hp = 100.;
                info!("God off");
            }
            Ok(())
        },
        "godenergy", "ge" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            if world.player.energy.max_energy.is_finite() {
                world.player.energy.max_energy = std::f32::INFINITY;
                world.player.energy.cur_energy = std::f32::INFINITY;
                info!("god-energy on");
            } else {
                world.player.energy.cur_energy = 100.;
                world.player.energy.max_energy = 100.;
                info!("God-energy off");
            }
            Ok(())
        },
        "godarmour", "ga" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            if world.player.health.armour.is_finite() {
                world.player.health.armour = std::f32::INFINITY;
                info!("Skin of steel");
            } else {
                world.player.health.armour = 5.;
                info!("Re-skin");
            }
            Ok(())
        },
        "elem" => {
            let world = gs.get_mut_world().ok_or(NoWorld)?;
            let &elem = args.get(1).ok_or(InvalidArg)?;
            let element = Element::get_from_str(elem).ok_or(NoSuchElement)?;

            let _= world.player.spell.add_element(element);
            Ok(())
        },
        "cmp" => {if let Content::Campaign(ref mut cmp) = state.content {
            if let Some(i) = args.get(1) {
                let i = i.parse().map_err(|_| InvalidArg)?;
                cmp.current = i;
                let lvl = cmp.next_level().ok_or(NoSuchLevel)?;
                
                let (health, energy, spell) = if let Some(world) = gs.get_world() {
                    (world.player.health, world.player.energy, world.player.spell.clone())
                } else {
                    (Health::default(), Energy::default(), Default::default())
                };
                state.switch(StateSwitch::PlayWith{health, energy, spell, lvl: Box::new(lvl)});
            } else {
                info!("{} levels. Current is {}", cmp.levels.len(), cmp.current);
            }
            Ok(())
        } else {
            Err(NoCampaign)
        }},
        "hello" => {
            info!("Hello!");
            Ok(())
        },
        "quit" => {
            ctx.continuing = false;
            Ok(())
        },
        "strs" => {
            dbg_strs();
            Ok(())
        },
    }
}