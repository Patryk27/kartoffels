use crate::{cmd, rdi, wri, MEM_BLUETOOTH};

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
pub fn write_bluetooth_buffer_full(message: [u8; 32]) {
    for (i, byte) in message.iter().enumerate() {
        wri(MEM_BLUETOOTH, 0, cmd(0x02, i as u8, *byte, 0x00));
    }
}
//this one as well
#[inline(always)]
pub fn write_bluetooth_buffer_byte(data: u8, addr: usize) {
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
pub fn remove_buffer_front() {
    wri(MEM_BLUETOOTH, 0, cmd(0x03, 0x00, 0x00, 0x00));
}

#[inline(always)]
pub fn read_buffer_byte(address: usize) -> Option<u8> {
    if address >= 32 {
        return None;
    }
    let val = rdi(MEM_BLUETOOTH, address);
    let byte_array = val.to_le_bytes();
    if byte_array[0] == 0 {
        return None;
    }
    Some(byte_array[2])
}

#[inline(always)]
pub fn read_buffer() -> Option<[u8; 32]> {
    read_buffer_byte(0)?;
    let mut out = [0u8; 32];
    for addr in 0..32 {
        out[addr] = read_buffer_byte(addr).unwrap();
    }
    remove_buffer_front();
    Some(out)
}
// this is good :)
#[inline(always)]
pub fn is_buffer_empty() -> bool {
    read_buffer_byte(0).is_some()
}

#[inline(always)]
pub fn bluetooth_send_3x3(message: [u8; 32]) {
    write_bluetooth_buffer_full(message);
    send_bluetooth(3);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_5x5(message: [u8; 32]) {
    write_bluetooth_buffer_full(message);
    send_bluetooth(5);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_7x7(message: [u8; 32]) {
    write_bluetooth_buffer_full(message);
    send_bluetooth(7);
    clear_bluetooth_send_buffer();
}

#[inline(always)]
pub fn bluetooth_send_9x9(message: [u8; 32]) {
    write_bluetooth_buffer_full(message);
    send_bluetooth(9);
    clear_bluetooth_send_buffer();
}
