// MakAir Telemetry
//
// Copyright: 2020, Makers For Life
// License: Public Domain License

use rand::Rng;

use crate::control::*;

pub fn gen_random_message() -> ControlMessage {
    rand::thread_rng().gen()
}

pub fn gen_random_message_bytes() -> Vec<u8> {
    rand::thread_rng()
        .gen::<ControlMessage>()
        .to_control_frame()
}

pub fn gen_random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(1..=10);
    let mut bytes: Vec<u8> = vec![];
    for _ in 0..len {
        bytes.push(rng.gen());
    }
    bytes
}

pub fn gen_random_message_with_wrong_crc() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let msg = rng.gen::<ControlMessage>();
    msg.to_control_frame_with(Some(rng.gen()))
}
