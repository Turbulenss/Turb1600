#!/usr/bin/env python3
# =========================================================
#   turb1600 — Python Reference (v0.2.0 compatible)
# =========================================================

from typing import List

# =========================================================
# Parameters
# =========================================================

STATE_WORDS = 25
RATE_BYTES = 136
RATE_WORDS = RATE_BYTES // 8

FULL_ROUNDS = 32
FINAL_ROUNDS = 4
OUTPUT_BYTES = 128

DOMAIN_TAG = (
    b"turb1600:v2|state=1600|rate=1088|capacity=512|out=1024"
)

MASK = (1 << 64) - 1

# =========================================================
# Helpers
# =========================================================

def u64(x: int) -> int:
    return x & MASK

def rol(x: int, r: int) -> int:
    return u64((x << r) | (x >> (64 - r)))

def rot(round_: int, base: int) -> int:
    return (base + ((round_ * 13) & 63)) & 63

# =========================================================
# Round constant (matches Rust exactly)
# =========================================================

def round_constant(r: int) -> int:
    x = (
        r
        ^ 0x243F6A8885A308D3
        ^ rol(r, 17)
    )
    x ^= x >> 30
    x = u64(x * 0xBF58476D1CE4E5B9)
    x ^= x >> 27
    x = u64(x * 0x94D049BB133111EB)
    x ^= x >> 31
    return u64(x)

# =========================================================
# Permutation tables
# =========================================================

RHO = [
     0,  1, 62, 28, 27,
    36, 44,  6, 55, 20,
     3, 10, 43, 25, 39,
    41, 45, 15, 21,  8,
    18,  2, 61, 56, 14,
]

PI = [
     0,  7, 14, 21,  3,
    10, 17, 24,  6, 13,
    20,  2,  9, 16, 23,
     5, 12, 19,  1,  8,
    15, 22,  4, 11, 18,
]

# =========================================================
# Core operations
# =========================================================

def absorb_block(state: List[int], block: bytes) -> None:
    for i in range(RATE_WORDS):
        state[i] ^= int.from_bytes(block[i*8:(i+1)*8], "little")

def permute(state: List[int], tmp: List[int], r: int) -> None:
    # ---- theta ----
    c = [0]*5
    for x in range(5):
        c[x] = (
            state[x] ^
            state[x+5] ^
            state[x+10] ^
            state[x+15] ^
            state[x+20]
        )

    d = [
        c[4] ^ rol(c[1], 1),
        c[0] ^ rol(c[2], 1),
        c[1] ^ rol(c[3], 1),
        c[2] ^ rol(c[4], 1),
        c[3] ^ rol(c[0], 1),
    ]

    for y in range(5):
        for x in range(5):
            state[x + 5*y] ^= d[x]

    # ---- rho + pi ----
    for i in range(25):
        tmp[PI[i]] = rol(state[i], rot(r, RHO[i]))

    state[:] = tmp[:]

    # ---- chi ----
    for i in range(0, 25, 5):
        a, b, c, d_, e = state[i:i+5]
        state[i+0] ^= (~b) & c
        state[i+1] ^= (~c) & d_
        state[i+2] ^= (~d_) & e
        state[i+3] ^= (~e) & a
        state[i+4] ^= (~a) & b

    # ---- iota ----
    state[(r * 7) % STATE_WORDS] ^= round_constant(r)

    for i in range(25):
        state[i] &= MASK

# =========================================================
# Initialization
# =========================================================

def initialize_state() -> List[int]:
    state = [0]*STATE_WORDS
    block = bytearray(RATE_BYTES)

    n = min(len(DOMAIN_TAG), RATE_BYTES)
    block[:n] = DOMAIN_TAG[:n]
    block[n] = 0x01
    block[-1] |= 0x80

    absorb_block(state, block)

    tmp = [0]*STATE_WORDS
    for r in range(8):
        permute(state, tmp, r)

    return state

# =========================================================
# Public hash
# =========================================================

def turb1600_hash(message: bytes) -> bytes:
    state = initialize_state()
    tmp = [0]*STATE_WORDS
    round_ = 0

    # ---- absorb ----
    offset = 0
    while offset + RATE_BYTES <= len(message):
        absorb_block(state, message[offset:offset+RATE_BYTES])
        for _ in range(FULL_ROUNDS):
            permute(state, tmp, round_)
            round_ += 1
        offset += RATE_BYTES

    # ---- final block ----
    last = bytearray(RATE_BYTES)
    rem = len(message) - offset
    last[:rem] = message[offset:]
    last[rem] = 0x01
    last[-1] |= 0x80

    absorb_block(state, last)

    for _ in range(FULL_ROUNDS + FINAL_ROUNDS):
        permute(state, tmp, round_)
        round_ += 1

    # ---- squeeze ----
    out = bytearray()
    while len(out) < OUTPUT_BYTES:
        state[24] ^= MASK

        for i in range(RATE_WORDS):
            if len(out) >= OUTPUT_BYTES:
                break
            out.extend(state[i].to_bytes(8, "little"))

        permute(state, tmp, round_)
        round_ += 1

    return bytes(out[:OUTPUT_BYTES])

# =========================================================
# Self-test
# =========================================================
KATS = [
    b"",
    b"a",
    b"abc",
    b"message digest",
    b"abcdefghijklmnopqrstuvwxyz",
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    b"\x00",
    b"\x00" * 7,
    b"\x00" * 8,
    b"\x00" * 135,          # rate - 1
    b"\x00" * 136,          # exact rate
    b"\x00" * 137,          # rate + 1
    b"turb1600",
    bytes(range(256)),      # full byte spectrum
]

def generate_kats():
    print("turb1600 v0.2.0 KATs\n")
    for i, msg in enumerate(KATS):
        h = turb1600_hash(msg).hex()
        label = f"KAT-{i:02d}"
        print(f"{label}:")
        print(f"  msg = {msg!r}")
        print(f"  hash = {h}\n")

if __name__ == "__main__":
    d1 = turb1600_hash(b"abc")
    d2 = turb1600_hash(b"abc")
    assert d1 == d2
    print("PASS")
    print(d1.hex())
    generate_kats()
