// MakAir
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use rand::Rng;

use crate::control::*;

pub fn gen_random_message() -> ControlMessage {
    rand::thread_rng().gen()
}
