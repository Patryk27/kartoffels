use core::num::NonZeroU64;

use crate::{cmd, rdi, wri, MEM_BLUETOOTH};

pub fn rdi_wrapper(id: usize) -> u32 {
    rdi(MEM_BLUETOOTH, id)
}

#[derive(Debug)]
pub struct Message {
    pub message: [u8; 32],
    pub sender_id: NonZeroU64,
}

#[inline(always)]
pub fn is_bluetooth_ready() -> bool {
    rdi(MEM_BLUETOOTH, 0) == 1
}

#[inline(always)]
pub fn bluetooth_wait() {
    while !is_bluetooth_ready() {
        //
    }
}

#[inline(always)]
pub fn send_bluetooth(r: u8) {
    wri(MEM_BLUETOOTH, 0, cmd(0x01, r, 0x00, 0x00));
}

// these functions need better names
#[inline(always)]
pub fn write_bluetooth_send_buffer(message: [u8; 32]) {
    for (i, byte) in message.iter().enumerate() {
        wri(MEM_BLUETOOTH, 0, cmd(0x02, i as u8, *byte, 0x00));
    }
}
//this one as well
#[inline(always)]
pub fn write_bluetooth_send_buffer_byte(data: u8, addr: usize) {
    if addr >= 32 {
        return;
    }
    wri(MEM_BLUETOOTH, 0, cmd(0x02, addr as u8, data, 0x00));
}

// this one as well << extra infact
#[inline(always)]
pub fn clear_bluetooth_send_buffer() {
    wri(MEM_BLUETOOTH, 0, cmd(0x03, 0x00, 0x00, 0x00));
}

// I think we need to change calling both send and recieve buffers
#[inline(always)]
pub fn remove_read_buffer_front() {
    wri(MEM_BLUETOOTH, 0, cmd(0x04, 0x00, 0x00, 0x00));
}

pub fn pop_bluetooth_buffer() -> Option<Message> {
    let [front, _, empty, bytes_per] = rdi(MEM_BLUETOOTH, 1).to_le_bytes();
    if empty == 1 {
        return None;
    }
    let start_byte = 1 + ((front * bytes_per) / 4);
    let message = _bluetooth_message(start_byte as usize, bytes_per as usize);
    remove_read_buffer_front();
    Some(message)
}

#[inline(always)]
pub fn get_bluetooth_buffer(index: usize) -> Option<Message> {
    let [front, length, empty, bytes_per] = rdi(MEM_BLUETOOTH, 1).to_le_bytes();
    if empty == 1 || index > (length - 1) as usize {
        return None;
    }
    let start_byte = 1 + (((front + index as u8) % 5 * bytes_per) / 4);
    Some(_bluetooth_message(start_byte as usize, bytes_per as usize))
}

#[inline(always)]
pub fn _bluetooth_message(start_byte: usize, bytes_per: usize) -> Message {
    let mut out: [u8; 32] = [0; 32];
    let mut d1: u64 = 0;
    let mut d2: u64 = 0;
    for off in 0..(bytes_per / 4) {
        // this is hard coded atm to the data in kartoffels_world/bot/bluetooth.rs, maybe we should serde this or something
        match off {
            0..=7 => {
                let byte_group =
                    rdi(MEM_BLUETOOTH, 1 + start_byte + off).to_le_bytes();
                (0..4).for_each(|v| out[(off * 4) + v] = byte_group[v])
            }
            8 => {
                d1 = rdi(MEM_BLUETOOTH, 1 + start_byte + off) as u64;
            }
            9 => {
                d2 = rdi(MEM_BLUETOOTH, 1 + start_byte + off) as u64;
            }
            _ => {}
        }
    }
    Message {
        message: out,
        sender_id: NonZeroU64::new((d1 << 32) | d2).unwrap(),
    }
}
#[inline(always)]
pub fn peek_bluetooth_buffer() -> Option<Message> {
    // let's get the important info here
    let [front, _, empty, bytes_per] = rdi(MEM_BLUETOOTH, 1).to_le_bytes();
    if empty == 1 {
        return None;
    }
    let start_byte = 1 + ((front * bytes_per) / 4);
    Some(_bluetooth_message(start_byte as usize, bytes_per as usize))
}

#[inline(always)]
pub fn is_read_buffer_empty() -> bool {
    let [_, _, is_empty, _] = rdi(MEM_BLUETOOTH, 1).to_le_bytes();
    is_empty != 0
}

#[inline(always)]
pub fn bluetooth_send_3x3(message: [u8; 32]) {
    write_bluetooth_send_buffer(message);
    send_bluetooth(3);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_5x5(message: [u8; 32]) {
    write_bluetooth_send_buffer(message);
    send_bluetooth(5);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_7x7(message: [u8; 32]) {
    write_bluetooth_send_buffer(message);
    send_bluetooth(7);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_9x9(message: [u8; 32]) {
    write_bluetooth_send_buffer(message);
    send_bluetooth(9);
    clear_bluetooth_send_buffer();
}
