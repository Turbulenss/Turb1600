#!/usr/bin/env python3
# =========================================================
# turb1600 — Sponge-Based Hash Engine
# State: 1600-bit (25×64)
# Rate: 1088-bit | Capacity: 512-bit
# Output: 1024-bit
# =========================================================

import sys
from typing import List

# =========================================================
# Core parameters
# =========================================================

LANES = 25                     # 1600-bit state
BLOCK_BYTES = 136               # 1088-bit rate
BLOCK_LANES = BLOCK_BYTES // 8

ROUNDS_MAIN = 36                # increased diffusion
ROUNDS_FINAL = 6                # stronger finalization
OUT_BYTES = 128                 # 1024-bit output

INIT_TAG = b"turb1600|sponge|1600|1088|512|1024|release"
MASK = (1 << 64) - 1

# =========================================================
# Rotation helpers
# =========================================================

def rol(x: int, r: int) -> int:
    return ((x << r) | (x >> (64 - r))) & MASK

def rot_offset(round_: int, base: int) -> int:
    return (base + ((round_ * 13) & 63)) & 63

# =========================================================
# Round constant
# =========================================================

def round_constant(idx: int) -> int:
    x = idx ^ 0xA5A5A5A5A5A5A5A5 ^ rol(idx, 23)
    x ^= x >> 33
    x = (x * 0xC2B2AE3D27D4EB4F) & MASK
    x ^= x >> 29
    x = (x * 0x165667B19E3779F9) & MASK
    x ^= x >> 32
    return x & MASK

# =========================================================
# Permutation tables
# =========================================================

ROT_TABLE = [
    0,1,62,28,27,36,44,6,55,20,3,10,43,25,39,
    41,45,15,21,8,18,2,61,56,14
]

PERM_TABLE = [
    0,7,14,21,3,10,17,24,6,13,20,2,9,16,23,
    5,12,19,1,8,15,22,4,11,18
]

# =========================================================
# Core functions
# =========================================================

def absorb_block(state: List[int], block: bytes) -> None:
    for i in range(BLOCK_LANES):
        state[i] ^= int.from_bytes(block[i*8:(i+1)*8], "little")

def permute(state: List[int], tmp: List[int], round_: int) -> None:
    # Column mix (theta)
    c = [state[x] ^ state[x+5] ^ state[x+10] ^ state[x+15] ^ state[x+20] for x in range(5)]
    d = [c[4] ^ rol(c[1],1), c[0] ^ rol(c[2],1), c[1] ^ rol(c[3],1),
         c[2] ^ rol(c[4],1), c[3] ^ rol(c[0],1)]
    for y in range(5):
        for x in range(5):
            state[x + 5*y] ^= d[x]

    # Rho + Pi
    for i in range(LANES):
        tmp[PERM_TABLE[i]] = rol(state[i], rot_offset(round_, ROT_TABLE[i]))
    state[:] = tmp[:]

    # Chi
    for i in range(0, LANES, 5):
        a,b,c,d,e = state[i:i+5]
        state[i+0] ^= (~b) & c
        state[i+1] ^= (~c) & d
        state[i+2] ^= (~d) & e
        state[i+3] ^= (~e) & a
        state[i+4] ^= (~a) & b
        for j in range(5):
            state[i+j] &= MASK

    # Iota
    state[(round_ * 7) % LANES] ^= round_constant(round_)

# =========================================================
# State initialization
# =========================================================

def seed_state(tmp: List[int]) -> List[int]:
    state = [0] * LANES
    block = bytearray(BLOCK_BYTES)
    n = min(len(INIT_TAG), BLOCK_BYTES)
    block[:n] = INIT_TAG[:n]
    block[n] = 0x01
    block[-1] |= 0x80
    absorb_block(state, block)
    for r in range(8):
        permute(state, tmp, r)
    return state

# =========================================================
# Public hash
# =========================================================

def turb1600_hash(data: bytes) -> bytes:
    tmp = [0] * LANES
    state = seed_state(tmp)
    round_ = 0
    pos = 0

    # Absorb full blocks
    while pos + BLOCK_BYTES <= len(data):
        absorb_block(state, data[pos:pos+BLOCK_BYTES])
        for _ in range(ROUNDS_MAIN):
            permute(state, tmp, round_)
            round_ += 1
        pos += BLOCK_BYTES

    # Final block
    tail = bytearray(BLOCK_BYTES)
    rem = len(data) - pos
    tail[:rem] = data[pos:]
    tail[rem] = 0x01
    tail[-1] |= 0x80
    absorb_block(state, tail)

    for _ in range(ROUNDS_MAIN + ROUNDS_FINAL):
        permute(state, tmp, round_)
        round_ += 1

    # Squeeze
    out = bytearray()
    while len(out) < OUT_BYTES:
        state[-1] ^= MASK
        for i in range(BLOCK_LANES):
            if len(out) >= OUT_BYTES:
                break
            out.extend(state[i].to_bytes(8, "little"))
        permute(state, tmp, round_)
        round_ += 1

    return bytes(out[:OUT_BYTES])

# =========================================================
# CLI / self-test
# =========================================================

def print_hex(data: bytes) -> None:
    print("".join(f"{b:02x}" for b in data))

def usage() -> None:
    print(
        "Usage:\n"
        "  turb1600 <string>                 Hash a string\n"
        "  turb1600 --hex <hex-string>       Hash raw bytes from hex\n"
        "  turb1600 --file <path>            Hash file contents\n"
        "  turb1600 --tag <tag> <string>     Hash string with domain tag\n"
        "Options:\n"
        "  --raw                              Output raw bytes instead of hex"
    )
    sys.exit(1)

if __name__ == "__main__":
    # If no arguments, run KAT / self-test
    if len(sys.argv) == 1:
        tests = [
            b"", b"a", b"abc", b"message digest",
            b"abcdefghijklmnopqrstuvwxyz",
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            bytes(range(256))
        ]
        for msg in tests:
            h = turb1600_hash(msg)
            print(f"{msg!r} -> {h.hex()}")
        sys.exit(0)

    # --- CLI mode ---
    args = sys.argv[1:]
    raw_output = False
    if args[0] == "--raw":
        raw_output = True
        args = args[1:]
        if not args:
            usage()

    if not args:
        usage()

    if args[0] == "--hex":
        if len(args) < 2:
            usage()
        input_bytes = bytes.fromhex(args[1])
    elif args[0] == "--file":
        if len(args) < 2:
            usage()
        with open(args[1], "rb") as f:
            input_bytes = f.read()
    elif args[0] == "--tag":
        if len(args) < 3:
            usage()
        tag = args[1].encode()
        msg = args[2].encode()
        input_bytes = tag + b"\x00" + msg
    else:
        input_bytes = args[0].encode()

    out = turb1600_hash(input_bytes)
    if raw_output:
        sys.stdout.buffer.write(out)
    else:
        print_hex(out)
