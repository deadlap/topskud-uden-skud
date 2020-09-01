use crate::util::{sstr, add_sstr, Sstr};

use lazy_static::lazy_static;

use std::fs::File;
use std::io::Read;
use std::num::NonZeroU16;
use std::collections::HashMap;
use std::f32::consts::PI;

#[inline]
fn def_impact() -> Sstr {
    add_sstr("impact")
}

const DEG2RAD: f32 = PI / 180.;