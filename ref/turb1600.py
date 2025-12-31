#!/usr/bin/env python3
# =========================================================
#   turb1600 (REFERENCE IMPLEMENTATION)
# =========================================================

from typing import List

# =========================================================
#   SPONGE PARAMETERS
# =========================================================

WORD_BITS = 64
MASK = (1 << 64) - 1

WORDS = 25                  # 1600-bit state
RATE_BYTES = 136            # 1088-bit rate
RATE_WORDS = RATE_BYTES // 8

ROUNDS = 16
FINAL_ROUNDS = 4
OUTPUT_BYTES = 128          # 1024-bit output

_SEED_STRING = (
    b"turb1600 | sponge-hash | state=1600 | rate=1088 | "
    b"capacity=512 | output=1024 | v1"
)

# =========================================================
#   U64 HELPERS
# =========================================================

def u64(x: int) -> int:
    return x & MASK

def rol(x: int, r: int) -> int:
    return u64((x << r) | (x >> (64 - r)))

# =========================================================
#   ROUND CONSTANTS
# =========================================================

_RC_COUNT = 1024

def _gen_round_constants(n: int) -> List[int]:
    rc = []
    x = 0x9E3779B97F4A7C15
    for _ in range(n):
        x ^= (x << 7) & MASK
        x ^= (x >> 9)
        x ^= (x << 8) & MASK
        rc.append(x & MASK)
    return rc

ROUND_CONSTANTS = _gen_round_constants(_RC_COUNT)

# =========================================================
#   STATE INITIALIZATION
# =========================================================

def initialize_state() -> List[int]:
    state = [0] * WORDS
    for i, b in enumerate(_SEED_STRING):
        state[i % WORDS] ^= u64(b + i)
    return state

# =========================================================
#   ABSORB
# =========================================================

def absorb_block(state: List[int], block: bytes) -> None:
    for i in range(0, len(block), 8):
        state[i // 8] ^= u64(int.from_bytes(block[i:i+8], "little"))

# =========================================================
#   ROUND LAYERS
# =========================================================

def theta_diffusion(state: List[int]) -> List[int]:
    C = [0] * 5
    D = [0] * 5

    for x in range(5):
        C[x] = (
            state[x] ^
            state[x + 5] ^
            state[x + 10] ^
            state[x + 15] ^
            state[x + 20]
        )

    for x in range(5):
        D[x] = C[(x - 1) % 5] ^ rol(C[(x + 1) % 5], 1)

    out = state.copy()
    for x in range(5):
        for y in range(5):
            out[x + 5*y] ^= D[x]

    return [u64(x) for x in out]

_ROT = [
     0,  1, 62, 28, 27,
    36, 44,  6, 55, 20,
     3, 10, 43, 25, 39,
    41, 45, 15, 21,  8,
    18,  2, 61, 56, 14,
]

def bit_permutation(state: List[int]) -> List[int]:
    out = [0] * WORDS
    for i in range(WORDS):
        out[(i * 7) % WORDS] = rol(state[i], _ROT[i])
    return out

def chi_non_linearity(state: List[int]) -> List[int]:
    out = state.copy()
    for i in range(0, WORDS, 5):
        a, b, c, d, e = state[i:i+5]
        out[i+0] ^= (~b) & c
        out[i+1] ^= (~c) & d
        out[i+2] ^= (~d) & e
        out[i+3] ^= (~e) & a
        out[i+4] ^= (~a) & b
    return [u64(x) for x in out]

def add_round_constant(state: List[int], rc: int) -> None:
    state[0] ^= rc

# =========================================================
#   ROUND FUNCTION
# =========================================================

def round_function(state: List[int], r: int) -> List[int]:
    state = theta_diffusion(state)
    state = bit_permutation(state)
    state = chi_non_linearity(state)
    add_round_constant(state, ROUND_CONSTANTS[r])
    return state

# =========================================================
#   SQUEEZE
# =========================================================

def squeeze(state: List[int], out_bytes: int) -> bytes:
    out = bytearray()
    r = 0
    while len(out) < out_bytes:
        for w in state[:RATE_WORDS]:
            out.extend(w.to_bytes(8, "little"))
        state = round_function(state, r)
        r += 1
    return bytes(out[:out_bytes])

# =========================================================
#   TOP-LEVEL HASH
# =========================================================

def turb1600_hash(message: bytes) -> bytes:
    state = initialize_state()

    padlen = (-len(message) - 2) % RATE_BYTES
    padded = message + b'\x01' + b'\x00' * padlen + b'\x80'

    r = 0
    for i in range(0, len(padded), RATE_BYTES):
        absorb_block(state, padded[i:i + RATE_BYTES])
        for _ in range(ROUNDS):
            state = round_function(state, r)
            r += 1

    for _ in range(FINAL_ROUNDS):
        state = round_function(state, r)
        r += 1

    return squeeze(state, OUTPUT_BYTES)

# =========================================================
#   SELF-TEST
# =========================================================

def run_selftest():
    print("turb1600 reference self-test")
    d1 = turb1600_hash(b"abc")
    d2 = turb1600_hash(b"abc")
    assert d1 == d2
    print("PASS")
    print(d1.hex())

if __name__ == "__main__":
    run_selftest()
