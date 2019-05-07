#![no_main]

use optee_utee::{
    ta_close_session, ta_create, ta_destroy, ta_invoke_command, ta_open_session, trace_println,
};
use optee_utee::{AlgorithmId, Mac};
use optee_utee::{Attribute, AttributeId, TransientObject, TransientObjectType};
use optee_utee::{Error, ErrorKind, Parameters, Result};
use proto::Command;

pub const SHA1_HASH_SIZE: usize = 20;
pub const MAX_KEY_SIZE: usize = 64;
pub const MIN_KEY_SIZE: usize = 10;
pub const DBC2_MODULO: u32 = 1000000;

pub struct HmacOtp {
    pub counter: [u8; 8],
    pub key: [u8; MAX_KEY_SIZE],
    pub key_len: usize,
}

#[ta_create]
fn create() -> Result<()> {
    trace_println!("[+] TA create");
    Ok(())
}

#[ta_open_session]
fn open_session(_params: &mut Parameters, sess_ctx: *mut *mut HmacOtp) -> Result<()> {
    trace_println!("[+] TA open session");
    let ptr = Box::into_raw(Box::new(HmacOtp {
        counter: [0u8; 8],
        key: [0u8; MAX_KEY_SIZE],
        key_len: 0,
    }));
    unsafe {
        *sess_ctx = ptr;
    }
    Ok(())
}

#[ta_close_session]
fn close_session(sess_ctx: &mut HmacOtp) {
    unsafe { Box::from_raw(sess_ctx) };
    trace_println!("[+] TA close session");
}

#[ta_destroy]
fn destroy() {
    trace_println!("[+] TA destroy");
}

#[ta_invoke_command]
fn invoke_command(sess_ctx: &mut HmacOtp, cmd_id: u32, params: &mut Parameters) -> Result<()> {
    trace_println!("[+] TA invoke command");
    match Command::from(cmd_id) {
        Command::RegisterSharedKey => {
            return register_shared_key(sess_ctx, params);
        }
        Command::GetHOTP => {
            return get_hotp(sess_ctx, params);
        }
        _ => {
            return Err(Error::new(ErrorKind::BadParameters));
        }
    }
}

pub fn register_shared_key(hotp: &mut HmacOtp, params: &mut Parameters) -> Result<()> {
    let mut p = unsafe { params.0.as_memref().unwrap() };
    let buffer = p.buffer();
    hotp.key_len = buffer.len();
    hotp.key[..hotp.key_len].clone_from_slice(buffer);
    Ok(())
}

pub fn get_hotp(hotp: &mut HmacOtp, params: &mut Parameters) -> Result<()> {
    let mut mac: [u8; SHA1_HASH_SIZE] = [0x0; SHA1_HASH_SIZE];

    hmac_sha1(hotp, &mut mac)?;

    for i in (0..hotp.counter.len()).rev() {
        hotp.counter[i] += 1;
        if hotp.counter[i] > 0 {
            break;
        }
    }
    let hotp_val = truncate(&mut mac);
    let mut p = unsafe { params.0.as_value().unwrap() };
    p.set_a(hotp_val);
    Ok(())
}

pub fn hmac_sha1(hotp: &mut HmacOtp, out: &mut [u8]) -> Result<usize> {
    if hotp.key_len < MIN_KEY_SIZE || hotp.key_len > MAX_KEY_SIZE {
        return Err(Error::new(ErrorKind::BadParameters));
    }

    match Mac::allocate(AlgorithmId::HmacSha1, hotp.key_len * 8) {
        Err(e) => return Err(e),
        Ok(mac) => {
            match TransientObject::allocate(TransientObjectType::HmacSha1, hotp.key_len * 8) {
                Err(e) => return Err(e),
                Ok(mut key_object) => {
                    //KEY size can be larger than hotp.key_len
                    let mut tmp_key = hotp.key.to_vec();
                    tmp_key.truncate(hotp.key_len);
                    let attr = Attribute::from_ref(AttributeId::SecretValue, &mut tmp_key);
                    key_object.populate(&[attr])?;
                    mac.set_key(&key_object)?;
                }
            }
            mac.init(&[0u8; 0]);
            mac.update(&hotp.counter);
            let out_len = mac.compute_final(&[0u8; 0], out).unwrap();
            Ok(out_len)
        }
    }
}

pub fn truncate(hmac_result: &mut [u8]) -> u32 {
    let mut bin_code: u32;
    let offset: usize = (hmac_result[19] & 0xf) as usize;

    bin_code = ((hmac_result[offset] & 0x7f) as u32) << 24
        | ((hmac_result[offset + 1] & 0xff) as u32) << 16
        | ((hmac_result[offset + 2] & 0xff) as u32) << 8
        | ((hmac_result[offset + 3] & 0xff) as u32);

    bin_code %= DBC2_MODULO;
    return bin_code;
}

// TA configurations
const TA_FLAGS: u32 = 0;
const TA_DATA_SIZE: u32 = 32 * 1024;
const TA_STACK_SIZE: u32 = 2 * 1024;
const TA_VERSION: &[u8] = b"0.1\0";
const TA_DESCRIPTION: &[u8] = b"This is an HOTP example.\0";
const EXT_PROP_VALUE_1: &[u8] = b"HOTP TA\0";
const EXT_PROP_VALUE_2: u32 = 0x0010;
const TRACE_LEVEL: i32 = 4;
const TRACE_EXT_PREFIX: &[u8] = b"TA\0";
const TA_FRAMEWORK_STACK_SIZE: u32 = 2048;

include!(concat!(env!("OUT_DIR"), "/user_ta_header.rs"));
