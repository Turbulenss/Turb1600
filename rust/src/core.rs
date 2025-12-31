use std::sync::LazyLock;

// =========================================================
//   turb1600
// =========================================================

const MASK: u64 = 0xFFFF_FFFF_FFFF_FFFF;

const WORDS: usize = 25;                // 1600-bit state
const RATE_BYTES: usize = 136;           // 1088-bit rate
const RATE_WORDS: usize = RATE_BYTES / 8;

const ROUNDS: usize = 16;
const FINAL_ROUNDS: usize = 4;
const OUTPUT_BYTES: usize = 128;         // 1024-bit output

const SEED_STRING: &[u8] =
    b"turb1600 | sponge-hash | state=1600 | rate=1088 | capacity=512 | output=1024 | v1";



// =========================================================
//   U64 HELPERS
// =========================================================

#[inline(always)]
fn rol(x: u64, r: u32) -> u64 {
    x.rotate_left(r)
}

// =========================================================
//   ROUND CONSTANTS
// =========================================================

const RC_COUNT: usize = 1024;

fn gen_round_constants() -> [u64; RC_COUNT] {
    let mut rc = [0u64; RC_COUNT];
    let mut x: u64 = 0x9E3779B97F4A7C15;

    for i in 0..RC_COUNT {
        x ^= (x << 7) & MASK;
        x ^= x >> 9;
        x ^= (x << 8) & MASK;
        rc[i] = x & MASK;
    }
    rc
}

static ROUND_CONSTANTS: LazyLock<[u64; RC_COUNT]> =
    LazyLock::new(gen_round_constants);

// =========================================================
//   STATE INITIALIZATION
// =========================================================

fn initialize_state() -> [u64; WORDS] {
    let mut state = [0u64; WORDS];
    for (i, &b) in SEED_STRING.iter().enumerate() {
        state[i % WORDS] ^= (b as u64) + (i as u64);
    }
    state
}

// =========================================================
//   ABSORB
// =========================================================

fn absorb_block(state: &mut [u64; WORDS], block: &[u8]) {
    for i in (0..block.len()).step_by(8) {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&block[i..i + 8]);
        state[i / 8] ^= u64::from_le_bytes(buf);
    }
}

// =========================================================
//   ROUND LAYERS
// =========================================================

fn theta_diffusion(state: &[u64; WORDS]) -> [u64; WORDS] {
    let mut c = [0u64; 5];
    let mut d = [0u64; 5];

    for x in 0..5 {
        c[x] = state[x]
            ^ state[x + 5]
            ^ state[x + 10]
            ^ state[x + 15]
            ^ state[x + 20];
    }

    for x in 0..5 {
        d[x] = c[(x + 4) % 5] ^ rol(c[(x + 1) % 5], 1);
    }

    let mut out = *state;
    for x in 0..5 {
        for y in 0..5 {
            out[x + 5 * y] ^= d[x];
        }
    }

    for v in &mut out {
        *v &= MASK;
    }

    out
}

// Lane rotations
const ROT: [u32; WORDS] = [
     0,  1, 62, 28, 27,
    36, 44,  6, 55, 20,
     3, 10, 43, 25, 39,
    41, 45, 15, 21,  8,
    18,  2, 61, 56, 14,
];

fn bit_permutation(state: &[u64; WORDS]) -> [u64; WORDS] {
    let mut out = [0u64; WORDS];
    for i in 0..WORDS {
        out[(i * 7) % WORDS] = rol(state[i], ROT[i]);
    }
    out
}

fn chi_non_linearity(state: &[u64; WORDS]) -> [u64; WORDS] {
    let mut out = *state;

    for i in (0..WORDS).step_by(5) {
        let a = state[i];
        let b = state[i + 1];
        let c = state[i + 2];
        let d = state[i + 3];
        let e = state[i + 4];

        out[i]     ^= (!b) & c;
        out[i + 1] ^= (!c) & d;
        out[i + 2] ^= (!d) & e;
        out[i + 3] ^= (!e) & a;
        out[i + 4] ^= (!a) & b;
    }

    for v in &mut out {
        *v &= MASK;
    }

    out
}

fn add_round_constant(state: &mut [u64; WORDS], rc: u64) {
    state[0] ^= rc;
}

// =========================================================
//   ROUND FUNCTION
// =========================================================

fn round_function(state: &[u64; WORDS], r: usize) -> [u64; WORDS] {
    let mut s = theta_diffusion(state);
    s = bit_permutation(&s);
    s = chi_non_linearity(&s);
    add_round_constant(&mut s, ROUND_CONSTANTS[r % RC_COUNT]);
    s
}


// =========================================================
//   SQUEEZE
// =========================================================

fn squeeze(mut state: [u64; WORDS], out_bytes: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(out_bytes);
    let mut r = 0usize;

    while out.len() < out_bytes {
        for &w in &state[..RATE_WORDS] {
            out.extend_from_slice(&w.to_le_bytes());
        }
        state = round_function(&state, r);
        r += 1;
    }

    out.truncate(out_bytes);
    out
}

// =========================================================
//   TOP-LEVEL HASH
// =========================================================

pub fn turb1600_hash(message: &[u8]) -> Vec<u8> {
    let mut state = initialize_state();

    let padlen = (RATE_BYTES - (message.len() + 2) % RATE_BYTES) % RATE_BYTES;
    let mut padded = Vec::from(message);
    padded.push(0x01);
    padded.extend(vec![0u8; padlen]);
    padded.push(0x80);

    let mut r = 0usize;

    for block in padded.chunks(RATE_BYTES) {
        absorb_block(&mut state, block);
        for _ in 0..ROUNDS {
            state = round_function(&state, r);
            r += 1;
        }
    }

    for _ in 0..FINAL_ROUNDS {
        state = round_function(&state, r);
        r += 1;
    }

    squeeze(state, OUTPUT_BYTES)
}
