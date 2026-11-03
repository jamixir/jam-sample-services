// A JAM service whose refine hashes the payload ROUNDS times and whose accumulate stores the result.
// Everything JAM-specific follows GP v0.7.2: entry points (Appendix B), host calls fetch = 1 and
// write = 4, and the argument and operand encodings (serialization.tex).
#![no_std]
#![no_main]

use blake2b_simd::Params;

pub const ROUNDS: u32 = 1000;

#[polkavm_derive::polkavm_import]
extern "C" {
    #[polkavm_import(index = 1)]
    fn fetch(out: u64, offset: u64, len: u64, kind: u64, index: u64) -> u64;
    #[polkavm_import(index = 4)]
    fn write(key: u64, key_len: u64, value: u64, value_len: u64) -> u64;
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { core::arch::asm!("unimp", options(noreturn)) }
}

static mut OUTPUT: [u8; 32] = [0; 32];
static mut INPUT: [u8; 1024] = [0; 1024];

// GP general natural decoding: the count of leading 1 bits in the first byte is the tail length
fn natural(bytes: &[u8]) -> (u64, &[u8]) {
    let first = bytes[0];
    let l = first.leading_ones() as usize;

    if l == 8 {
        let mut v = [0u8; 8];
        v.copy_from_slice(&bytes[1..9]);
        return (u64::from_le_bytes(v), &bytes[9..]);
    }

    let mut tail = 0u64;
    for (k, b) in bytes[1..=l].iter().enumerate() {
        tail |= (*b as u64) << (8 * k);
    }

    let high = (first as u64) & ((1u64 << (7 - l)) - 1);
    (tail | (high << (8 * l)), &bytes[l + 1..])
}

fn blake2b_256(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(Params::new().hash_length(32).hash(data).as_bytes());
    out
}

// Formula (B.5) v0.7.2
// a = E(c, i, s, ↕payload, H(p)); the output is h_ROUNDS with h_0 = payload, h_k+1 = H(h_k)
#[polkavm_derive::polkavm_export]
extern "C" fn refine(args: u64, args_len: u64) -> (u64, u64) {
    let a = unsafe { core::slice::from_raw_parts(args as *const u8, args_len as usize) };
    let (_core, a) = natural(a);
    let (_item, a) = natural(a);
    let (_service, a) = natural(a);
    let (len, a) = natural(a);
    let payload = &a[..len as usize];

    let mut h = blake2b_256(payload);
    for _ in 1..ROUNDS {
        h = blake2b_256(&h);
    }

    unsafe {
        OUTPUT = h;
        (core::ptr::addr_of!(OUTPUT) as u64, 32)
    }
}

// Formula (B.9) v0.7.2
// a = E(t, s, |i|); input 0, fetched with kind 15, is E(0, package, segment root, authorizer,
// payload hash, gas, O(result), ↕trace); a 32-byte ok result is stored under its payload hash
// and yielded
#[polkavm_derive::polkavm_export]
extern "C" fn accumulate(_args: u64, _args_len: u64) -> (u64, u64) {
    unsafe {
        let buffer = core::ptr::addr_of_mut!(INPUT) as *mut u8;
        let len = fetch(buffer as u64, 0, 1024, 15, 0);
        if len as i64 <= 0 || len > 1024 {
            return (0, 0);
        }

        let input = core::slice::from_raw_parts(buffer, len as usize);
        // an operand tuple, not a deferred transfer
        if input[0] != 0 {
            return (0, 0);
        }

        let payload_hash = &input[1 + 96..1 + 128];
        let (_gas, rest) = natural(&input[1 + 128..]);
        // O(result): 0 then ↕blob for a blob
        if rest[0] != 0 {
            return (0, 0);
        }
        let (result_len, rest) = natural(&rest[1..]);
        if result_len != 32 {
            return (0, 0);
        }

        core::ptr::copy_nonoverlapping(rest.as_ptr(), core::ptr::addr_of_mut!(OUTPUT) as *mut u8, 32);
        let out = core::ptr::addr_of!(OUTPUT) as u64;
        write(payload_hash.as_ptr() as u64, 32, out, 32);
        (out, 32)
    }
}
