// =========================================================
// turb1600 — Optimized Sponge Hash
// State: 1600-bit | Rate: 1088 | Capacity: 512
// Output: 1024-bit | Version: v0.2.0 (fixed)
// =========================================================

use std::sync::LazyLock;

// =========================================================
// Parameters
// =========================================================

const STATE_WORDS: usize = 25;              // 1600-bit state
const RATE_BYTES: usize = 136;              // 1088-bit rate
const RATE_WORDS: usize = RATE_BYTES / 8;

const FULL_ROUNDS: usize = 32;
const FINAL_ROUNDS: usize = 4;
const OUTPUT_BYTES: usize = 128;            // 1024-bit output

// Domain separation / initialization tag
const DOMAIN_TAG: &[u8] =
    b"turb1600:v2|state=1600|rate=1088|capacity=512|out=1024";

// =========================================================
// Rotation helpers
// =========================================================

#[inline(always)]
fn rol(x: u64, r: u32) -> u64 {
    x.rotate_left(r)
}

#[inline(always)]
fn rot(round: usize, base: u32) -> u32 {
    base.wrapping_add(((round as u32) * 13) & 63)
}

// =========================================================
// Round constant generator (external, stateless)
// =========================================================

#[inline(always)]
fn round_constant(r: usize) -> u64 {
    let mut x = (r as u64)
        ^ 0x243F6A8885A308D3
        ^ ((r as u64).rotate_left(17));

    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58476D1CE4E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D049BB133111EB);
    x ^= x >> 31;
    x
}

// =========================================================
// Permutation tables
// =========================================================

const RHO: [u32; STATE_WORDS] = [
    0, 1, 62, 28, 27,
    36, 44, 6, 55, 20,
    3, 10, 43, 25, 39,
    41, 45, 15, 21, 8,
    18, 2, 61, 56, 14,
];

const PI: [usize; STATE_WORDS] = [
    0, 7, 14, 21, 3,
    10, 17, 24, 6, 13,
    20, 2, 9, 16, 23,
    5, 12, 19, 1, 8,
    15, 22, 4, 11, 18,
];

// =========================================================
// State initialization
// =========================================================

#[inline(always)]
fn initialize_state(tmp: &mut [u64; STATE_WORDS]) -> [u64; STATE_WORDS] {
    let mut state = [0u64; STATE_WORDS];
    let mut block = [0u8; RATE_BYTES];

    let n = DOMAIN_TAG.len().min(RATE_BYTES);
    block[..n].copy_from_slice(&DOMAIN_TAG[..n]);
    block[n] = 0x01;
    block[RATE_BYTES - 1] |= 0x80;

    absorb_block(&mut state, &block);

    for r in 0..8 {
        permute(&mut state, tmp, r);
    }

    state
}

// =========================================================
// Absorb (aligned, unchecked)
// =========================================================

#[inline(always)]
fn absorb_block(state: &mut [u64; STATE_WORDS], block: &[u8]) {
    unsafe {
        let s = state.as_mut_ptr();
        let p = block.as_ptr() as *const u64;

        let mut i = 0;
        while i < RATE_WORDS {
            *s.add(i) ^= u64::from_le(*p.add(i));
            i += 1;
        }
    }
}

// =========================================================
// Core permutation
// =========================================================

#[inline(always)]
fn permute(state: &mut [u64; STATE_WORDS], tmp: &mut [u64; STATE_WORDS], r: usize) {
    unsafe {
        let s = state.as_mut_ptr();

        // ---- theta ----
        let c0 = *s.add(0)  ^ *s.add(5)  ^ *s.add(10) ^ *s.add(15) ^ *s.add(20);
        let c1 = *s.add(1)  ^ *s.add(6)  ^ *s.add(11) ^ *s.add(16) ^ *s.add(21);
        let c2 = *s.add(2)  ^ *s.add(7)  ^ *s.add(12) ^ *s.add(17) ^ *s.add(22);
        let c3 = *s.add(3)  ^ *s.add(8)  ^ *s.add(13) ^ *s.add(18) ^ *s.add(23);
        let c4 = *s.add(4)  ^ *s.add(9)  ^ *s.add(14) ^ *s.add(19) ^ *s.add(24);

        let d0 = c4 ^ rol(c1, 1);
        let d1 = c0 ^ rol(c2, 1);
        let d2 = c1 ^ rol(c3, 1);
        let d3 = c2 ^ rol(c4, 1);
        let d4 = c3 ^ rol(c0, 1);

        let mut i = 0;
        while i < 25 {
            *s.add(i)     ^= d0;
            *s.add(i + 1) ^= d1;
            *s.add(i + 2) ^= d2;
            *s.add(i + 3) ^= d3;
            *s.add(i + 4) ^= d4;
            i += 5;
        }

        // ---- rho + pi ----
        i = 0;
        while i < 25 {
            *tmp.get_unchecked_mut(PI[i]) =
                rol(*s.add(i), rot(r, RHO[i]));
            i += 1;
        }

        *state = *tmp;
        let s = state.as_mut_ptr();

        // ---- chi ----
        i = 0;
        while i < 25 {
            let a = *s.add(i);
            let b = *s.add(i + 1);
            let c = *s.add(i + 2);
            let d = *s.add(i + 3);
            let e = *s.add(i + 4);

            *s.add(i)     ^= (!b) & c;
            *s.add(i + 1) ^= (!c) & d;
            *s.add(i + 2) ^= (!d) & e;
            *s.add(i + 3) ^= (!e) & a;
            *s.add(i + 4) ^= (!a) & b;

            i += 5;
        }

        // ---- iota ----
        *s.add((r * 7) % STATE_WORDS) ^= round_constant(r);
    }
}

// =========================================================
// Public hash API
// =========================================================

pub fn turb1600_hash(message: &[u8]) -> Vec<u8> {
    let mut tmp = [0u64; STATE_WORDS];
    let mut state = initialize_state(&mut tmp);
    let mut round = 0usize;

    // ---- absorb ----
    let mut offset = 0;
    while offset + RATE_BYTES <= message.len() {
        absorb_block(&mut state, &message[offset..offset + RATE_BYTES]);
        for _ in 0..FULL_ROUNDS {
            permute(&mut state, &mut tmp, round);
            round += 1;
        }
        offset += RATE_BYTES;
    }

    // ---- final block ----
    let mut last = [0u8; RATE_BYTES];
    let rem = message.len() - offset;
    last[..rem].copy_from_slice(&message[offset..]);
    last[rem] = 0x01;
    last[RATE_BYTES - 1] |= 0x80;

    absorb_block(&mut state, &last);

    for _ in 0..(FULL_ROUNDS + FINAL_ROUNDS) {
        permute(&mut state, &mut tmp, round);
        round += 1;
    }

    // ---- squeeze ----
    let mut out = vec![0u8; OUTPUT_BYTES];
    let mut o = 0;

    while o < OUTPUT_BYTES {
        state[24] ^= u64::MAX;

        let mut i = 0;
        while i < RATE_WORDS && o < OUTPUT_BYTES {
            let bytes = state[i].to_le_bytes();
            let n = (OUTPUT_BYTES - o).min(8);
            out[o..o + n].copy_from_slice(&bytes[..n]);
            o += n;
            i += 1;
        }

        permute(&mut state, &mut tmp, round);
        round += 1;
    }

    out
}
