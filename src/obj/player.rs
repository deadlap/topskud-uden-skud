use std::{option::IntoIter, iter::{Chain, IntoIterator}};

use ggez::{Context, GameResult, graphics::{self, WHITE, Color}};

use crate::{
    util::{Point2},
    io::{
        snd::MediaPlayer,
        tex::{Assets, },
    },
};

use super::{Object, energy::Energy, health::Health, spell::{SpellInstance, SPELLS, Element}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub obj: Object,
    #[serde(skip)]
    pub health: Health,
    #[serde(skip)]
    pub spell: ElemSlots,
    #[serde(skip)]
    pub energy: Energy,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveSlot {
    Slot = 0,
    Slot2 = 1,
    Slot3 = 2,
}

impl ActiveSlot {
    #[inline]
    fn subtract(&mut self) {
        use self::ActiveSlot::*;
        *self = match *self {
            Slot => Slot,
            Slot2 => Slot,
            Slot3 => Slot2,
        };
    }
}
impl Default for ActiveSlot {
    #[inline(always)]
    fn default() -> Self {
        ActiveSlot::Slot
    }
}

#[derive(Debug, Default, Clone)]
pub struct ElemSlots {
    pub cur_spell: Option<SpellInstance<'static>>,
    // pub cur_elements: Vec<Element>,
    pub active: ActiveSlot,
    pub slot: Option<Element>,
    pub slot2: Option<Element>,
    pub slot3: Option<Element>,
}

impl ElemSlots {
    #[inline(always)]
    pub fn slot_has_element(&self, new_active: ActiveSlot) -> bool {
        match new_active {
            ActiveSlot::Slot => self.slot.is_some(),
            ActiveSlot::Slot2 => self.slot2.is_some(),
            ActiveSlot::Slot3 => self.slot3.is_some(),
        }
    }
    pub fn init_active(&mut self) {
        self.active = match self {
            ElemSlots{slot: Some(_), ..} => ActiveSlot::Slot,
            ElemSlots{slot: None, slot2: Some(_), ..} => ActiveSlot::Slot2,
            ElemSlots{slot: None, slot2: None, slot3: Some(_), ..} => ActiveSlot::Slot3,
            ElemSlots{slot: None, slot2: None, slot3: None, ..} => ActiveSlot::Slot,
        };
    }
    #[inline(always)]
    pub fn switch(&mut self, new_active: ActiveSlot) {
        if self.slot_has_element(new_active) {
            self.active = new_active;
            if let Some(cur_spell) = self.get_cur_mut() {
                cur_spell.being_charged = false;
            }
            self.cur_spell = self.find_current_spell();
        }
    }
    #[must_use]
    pub fn take_active(&mut self) -> Option<Element> {
        let spe = match self.active {
            ActiveSlot::Slot => std::mem::take(&mut self.slot),
            ActiveSlot::Slot2 => std::mem::take(&mut self.slot2),
            ActiveSlot::Slot3 => std::mem::take(&mut self.slot3),
        };
        while !self.slot_has_element(self.active) {
            self.active.subtract();
        }
        spe
    }
    #[inline(always)]
    pub fn get_active_element(&self) -> Option<&Element> {
        match self.active {
            ActiveSlot::Slot => self.slot.as_ref(),
            ActiveSlot::Slot2 => self.slot2.as_ref(),
            ActiveSlot::Slot3 => self.slot3.as_ref(),
        }
    }
    #[inline(always)]
    pub fn get_active_element_mut(&mut self) -> Option<&mut Element> {
        match self.active {
            ActiveSlot::Slot => self.slot.as_mut(),
            ActiveSlot::Slot2 => self.slot2.as_mut(),
            ActiveSlot::Slot3 => self.slot3.as_mut(),
        }
    }
    #[must_use]
    pub fn insert(&mut self) -> &mut Option<Element> {
        match self {
            ElemSlots{slot: ref mut s @ None, ..} |
            ElemSlots{slot2: ref mut s @ None, ..} |
            ElemSlots{slot3: ref mut s @ None, ..} |
            ElemSlots{active: ActiveSlot::Slot, slot: ref mut s, ..} |
            ElemSlots{active: ActiveSlot::Slot2, slot2: ref mut s, ..} |
            ElemSlots{active: ActiveSlot::Slot3, slot3: ref mut s, ..} => s,
        }
    }
    #[must_use]
    #[inline]
    pub fn add_element(&mut self, element: Element) -> Option<Element> {
        std::mem::replace(self.insert(), Some(element))
    }
    pub fn find_current_spell(&mut self) -> Option<SpellInstance<'static>> {
        if let Some(active_element) = self.get_active_element() {
            if let Some((_, spell)) = SPELLS.iter().find(
            |&spe| spe.1.element_type.contains(active_element) 
            ) {
                return Some(spell.make_instance())
            }
        }
        None
    }
    #[inline(always)]
    pub fn get_cur(&self) -> Option<&SpellInstance<'static>> {
        self.cur_spell.as_ref()
    }
    #[inline(always)]
    pub fn get_cur_mut(&mut self) -> Option<&mut SpellInstance<'static>> {
        self.cur_spell.as_mut()
    }
}

impl IntoIterator for ElemSlots {
    type IntoIter = Chain<
        Chain<IntoIter<Element>, IntoIter<Element>>,
        IntoIter<Element>,
    >;
    type Item = <Self::IntoIter as Iterator>::Item;
    fn into_iter(self) -> Self::IntoIter {
        #[allow(clippy::unneeded_field_pattern)]
        let ElemSlots{cur_spell: _, active: _, slot, slot2, slot3} = self;

        slot.into_iter().chain(slot2).chain(slot3)
    }
}
impl Player {
    #[inline]
    pub fn new(obj: Object) -> Self {
        Self {
            obj,
            spell: Default::default(),
            health: Health::default(),
            energy: Energy::default(),
        }
    }
    #[inline]
    pub fn from_point(p: Point2) -> Self {
        Player::new(Object::new(p))
    }
    #[inline]
    pub fn with_health(self, health: Health) -> Self {
        Self {
            health,
            .. self
        }
    }
    #[inline]
    pub fn with_energy(self, energy: Energy) -> Self {
        Self {
            energy,
            .. self
        }
    }
    #[inline]
    pub fn with_spell(self, spell: ElemSlots) -> Self {
        Self {
            spell,
            .. self
        }
    }
    #[inline]
    pub fn draw_player(&self, ctx: &mut Context, a: &Assets) -> GameResult<()> {
        self.draw(ctx, a, "common/player", WHITE)
    }
    pub fn draw(&self, ctx: &mut Context, a: &Assets, sprite: &str, color: Color) -> GameResult<()> {
        {
            // let hands_sprite = if let Some(wep) = self.wep.get_active() {
            //     wep.weapon.hands_sprite
            // } else {
            //     "weapons/knife_hands"
            // };

            // let dp = graphics::DrawParam {
            //     dest: (self.obj.pos+angle_to_vec(self.obj.rot)*16.).into(),
            //     color,
            //     .. self.obj.drawparams()
            // };
            // let img = a.get_img(ctx, hands_sprite);
            // graphics::draw(ctx, &*img, dp)?;
        }
        let img = a.get_img(ctx, sprite);
        self.obj.draw(ctx, &*img, color)
    }
    pub fn update(&mut self, ctx: &mut Context, mplayer: &mut MediaPlayer) -> GameResult<()> {
        if self.spell.cur_spell.is_none() && self.spell.get_active_element().is_some() {
            self.spell.cur_spell = self.spell.find_current_spell();
        }
        if let Some(spell) = self.spell.get_cur_mut() {
            spell.update(ctx, mplayer)?;
        }
        self.energy.update();
        Ok(())
    }
}
