// =========================================================
// turb1600 — Sponge-Based Hash Engine
// State: 1600-bit (25×64)
// Rate: 1088-bit | Capacity: 512-bit
// Output: 1024-bit
// =========================================================

#![allow(clippy::needless_range_loop)]

// =========================================================
// Core parameters
// =========================================================

const LANES: usize = 25;                // 1600-bit state
const BLOCK_BYTES: usize = 136;         // 1088-bit rate
const BLOCK_LANES: usize = BLOCK_BYTES / 8;

const ROUNDS_MAIN: usize = 36;          // increased diffusion
const ROUNDS_FINAL: usize = 6;          // stronger finalization
const OUT_BYTES: usize = 128;           // 1024-bit output

// Domain separation seed
const INIT_TAG: &[u8] =
    b"turb1600|sponge|1600|1088|512|1024|release";

// =========================================================
// Rotation utilities
// =========================================================

#[inline(always)]
fn rotl(x: u64, r: u32) -> u64 {
    x.rotate_left(r)
}

#[inline(always)]
fn rot_offset(round: usize, base: u32) -> u32 {
    base.wrapping_add(((round as u32) * 13) & 63)
}

// =========================================================
// Dynamic round constant
// =========================================================

#[inline(always)]
fn round_constant(idx: usize) -> u64 {
    let mut x = (idx as u64)
        ^ 0xA5A5A5A5A5A5A5A5
        ^ ((idx as u64).rotate_left(23));

    x ^= x >> 33;
    x = x.wrapping_mul(0xC2B2AE3D27D4EB4F);
    x ^= x >> 29;
    x = x.wrapping_mul(0x165667B19E3779F9);
    x ^= x >> 32;
    x
}

// =========================================================
// Permutation tables
// =========================================================

const ROT_TABLE: [u32; LANES] = [
    0, 1, 62, 28, 27,
    36, 44, 6, 55, 20,
    3, 10, 43, 25, 39,
    41, 45, 15, 21, 8,
    18, 2, 61, 56, 14,
];

const PERM_TABLE: [usize; LANES] = [
    0, 7, 14, 21, 3,
    10, 17, 24, 6, 13,
    20, 2, 9, 16, 23,
    5, 12, 19, 1, 8,
    15, 22, 4, 11, 18,
];

// =========================================================
// State seeding
// =========================================================

#[inline(always)]
fn seed_state(tmp: &mut [u64; LANES]) -> [u64; LANES] {
    let mut s = [0u64; LANES];
    let mut buf = [0u8; BLOCK_BYTES];

    let n = INIT_TAG.len().min(BLOCK_BYTES);
    buf[..n].copy_from_slice(&INIT_TAG[..n]);
    buf[n] = 0x01;
    buf[BLOCK_BYTES - 1] |= 0x80;

    absorb_block(&mut s, &buf);

    for r in 0..8 {
        permute(&mut s, tmp, r);
    }

    s
}

// =========================================================
// Absorption
// =========================================================

#[inline(always)]
fn absorb_block(state: &mut [u64; LANES], block: &[u8]) {
    unsafe {
        let sp = state.as_mut_ptr();
        let bp = block.as_ptr() as *const u64;

        for i in 0..BLOCK_LANES {
            *sp.add(i) ^= u64::from_le(*bp.add(i));
        }
    }
}

// =========================================================
// Core permutation
// =========================================================

#[inline(always)]
fn permute(state: &mut [u64; LANES], tmp: &mut [u64; LANES], round: usize) {
    unsafe {
        let s = state.as_mut_ptr();

        // ---- column mixing ----
        let c = [
            *s.add(0) ^ *s.add(5) ^ *s.add(10) ^ *s.add(15) ^ *s.add(20),
            *s.add(1) ^ *s.add(6) ^ *s.add(11) ^ *s.add(16) ^ *s.add(21),
            *s.add(2) ^ *s.add(7) ^ *s.add(12) ^ *s.add(17) ^ *s.add(22),
            *s.add(3) ^ *s.add(8) ^ *s.add(13) ^ *s.add(18) ^ *s.add(23),
            *s.add(4) ^ *s.add(9) ^ *s.add(14) ^ *s.add(19) ^ *s.add(24),
        ];

        let d = [
            c[4] ^ rotl(c[1], 1),
            c[0] ^ rotl(c[2], 1),
            c[1] ^ rotl(c[3], 1),
            c[2] ^ rotl(c[4], 1),
            c[3] ^ rotl(c[0], 1),
        ];

        for i in (0..LANES).step_by(5) {
            *s.add(i)     ^= d[0];
            *s.add(i + 1) ^= d[1];
            *s.add(i + 2) ^= d[2];
            *s.add(i + 3) ^= d[3];
            *s.add(i + 4) ^= d[4];
        }

        // ---- rotation + permutation ----
        for i in 0..LANES {
            *tmp.get_unchecked_mut(PERM_TABLE[i]) =
                rotl(*s.add(i), rot_offset(round, ROT_TABLE[i]));
        }

        *state = *tmp;
        let s = state.as_mut_ptr();

        // ---- nonlinear layer ----
        for i in (0..LANES).step_by(5) {
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
        }

        // ---- round injection ----
        *s.add((round * 7) % LANES) ^= round_constant(round);
    }
}

// =========================================================
// Public hashing API
// =========================================================

pub fn turb1600_hash(data: &[u8]) -> Vec<u8> {
    let mut tmp = [0u64; LANES];
    let mut state = seed_state(&mut tmp);
    let mut round = 0usize;

    let mut pos = 0;
    while pos + BLOCK_BYTES <= data.len() {
        absorb_block(&mut state, &data[pos..pos + BLOCK_BYTES]);
        for _ in 0..ROUNDS_MAIN {
            permute(&mut state, &mut tmp, round);
            round += 1;
        }
        pos += BLOCK_BYTES;
    }

    let mut tail = [0u8; BLOCK_BYTES];
    let rem = data.len() - pos;
    tail[..rem].copy_from_slice(&data[pos..]);
    tail[rem] = 0x01;
    tail[BLOCK_BYTES - 1] |= 0x80;

    absorb_block(&mut state, &tail);

    for _ in 0..(ROUNDS_MAIN + ROUNDS_FINAL) {
        permute(&mut state, &mut tmp, round);
        round += 1;
    }

    let mut out = vec![0u8; OUT_BYTES];
    let mut off = 0;

    while off < OUT_BYTES {
        state[LANES - 1] ^= u64::MAX;

        for i in 0..BLOCK_LANES {
            if off >= OUT_BYTES {
                break;
            }
            let bytes = state[i].to_le_bytes();
            let n = (OUT_BYTES - off).min(8);
            out[off..off + n].copy_from_slice(&bytes[..n]);
            off += n;
        }

        permute(&mut state, &mut tmp, round);
        round += 1;
    }

    out
}
