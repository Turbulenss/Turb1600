use std::sync::LazyLock;

// =========================================================
//   turb1600 — Optimized
// =========================================================

const WORDS: usize = 25;
const RATE_BYTES: usize = 136;
const RATE_WORDS: usize = RATE_BYTES / 8;

const ROUNDS: usize = 16;
const FINAL_ROUNDS: usize = 4;
const OUTPUT_BYTES: usize = 128;

const SEED_STRING: &[u8] =
    b"turb1600 | sponge-hash | state=1600 | rate=1088 | capacity=512 | output=1024 | v1";

// =========================================================
//   helpers
// =========================================================

#[inline(always)]
fn rol(x: u64, r: u32) -> u64 {
    x.rotate_left(r)
}

// =========================================================
//   round constants
// =========================================================

const RC_COUNT: usize = 1024;

static RC: LazyLock<[u64; RC_COUNT]> = LazyLock::new(|| {
    let mut rc = [0u64; RC_COUNT];
    let mut x = 0x9E3779B97F4A7C15u64;
    let mut i = 0;
    while i < RC_COUNT {
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        rc[i] = x;
        i += 1;
    }
    rc
});

// =========================================================
//   permutation tables
// =========================================================

const ROT: [u32; WORDS] = [
     0,  1, 62, 28, 27,
    36, 44,  6, 55, 20,
     3, 10, 43, 25, 39,
    41, 45, 15, 21,  8,
    18,  2, 61, 56, 14,
];

const PI: [usize; WORDS] = [
     0,  7, 14, 21,  3,
    10, 17, 24,  6, 13,
    20,  2,  9, 16, 23,
     5, 12, 19,  1,  8,
    15, 22,  4, 11, 18,
];

// =========================================================
//   state initialization
// =========================================================

#[inline(always)]
fn initialize_state() -> [u64; WORDS] {
    let mut s = [0u64; WORDS];
    let mut i = 0;
    while i < SEED_STRING.len() {
        s[i % WORDS] ^= (SEED_STRING[i] as u64) + i as u64;
        i += 1;
    }
    s
}

// =========================================================
//   absorb (aligned, unchecked)
// =========================================================

#[inline(always)]
fn absorb_block(state: &mut [u64; WORDS], block: &[u8]) {
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
//   permutation (fully unrolled)
// =========================================================

#[inline(always)]
fn permute(state: &mut [u64; WORDS], tmp: &mut [u64; WORDS], r: usize) {
    unsafe {
        let s = state.as_mut_ptr();

        // ---- theta ----
        let c0 = *s.add(0) ^ *s.add(5) ^ *s.add(10) ^ *s.add(15) ^ *s.add(20);
        let c1 = *s.add(1) ^ *s.add(6) ^ *s.add(11) ^ *s.add(16) ^ *s.add(21);
        let c2 = *s.add(2) ^ *s.add(7) ^ *s.add(12) ^ *s.add(17) ^ *s.add(22);
        let c3 = *s.add(3) ^ *s.add(8) ^ *s.add(13) ^ *s.add(18) ^ *s.add(23);
        let c4 = *s.add(4) ^ *s.add(9) ^ *s.add(14) ^ *s.add(19) ^ *s.add(24);

        let d0 = c4 ^ c1.rotate_left(1);
        let d1 = c0 ^ c2.rotate_left(1);
        let d2 = c1 ^ c3.rotate_left(1);
        let d3 = c2 ^ c4.rotate_left(1);
        let d4 = c3 ^ c0.rotate_left(1);

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
                (*s.add(i)).rotate_left(ROT[i]);
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
        *s ^= RC[r & (RC_COUNT - 1)];
    }
}

// =========================================================
//   hash
// =========================================================

pub fn turb1600_hash(message: &[u8]) -> Vec<u8> {
    let mut state = initialize_state();
    let mut tmp = [0u64; WORDS];
    let mut r = 0usize;

    // ---- absorb full blocks ----
    let mut offset = 0;
    while offset + RATE_BYTES <= message.len() {
        absorb_block(&mut state, &message[offset..offset + RATE_BYTES]);
        for _ in 0..ROUNDS {
            permute(&mut state, &mut tmp, r);
            r += 1;
        }
        offset += RATE_BYTES;
    }

    // ---- final padded block ----
    let mut last = [0u8; RATE_BYTES];
    let rem = message.len() - offset;
    last[..rem].copy_from_slice(&message[offset..]);
    last[rem] = 0x01;
    last[RATE_BYTES - 1] |= 0x80;

    absorb_block(&mut state, &last);

    for _ in 0..ROUNDS + FINAL_ROUNDS {
        permute(&mut state, &mut tmp, r);
        r += 1;
    }

    // ---- squeeze ----
    let mut out = vec![0u8; OUTPUT_BYTES];
    let mut o = 0;
    while o < OUTPUT_BYTES {
        let mut i = 0;
        while i < RATE_WORDS && o < OUTPUT_BYTES {
            let bytes = state[i].to_le_bytes();
            let n = (OUTPUT_BYTES - o).min(8);
            out[o..o + n].copy_from_slice(&bytes[..n]);
            o += n;
            i += 1;
        }
        permute(&mut state, &mut tmp, r);
        r += 1;
    }

    out
}
