# turb1600

turb1600 is an experimental sponge-based cryptographic hash function built around a 1600-bit internal state and producing a 1024-bit output.

The design is inspired by Keccak-style permutations but is **not compatible with SHA-3** and is **not standardized or audited**.

This repository contains:
- A Python reference implementation
- A Rust reference implementation
- A Rust command-line interface (CLI)

> **Warning**
>
> This project is experimental and unaudited.
> It is intended strictly for learning, research, and experimentation.
> **Do not use this code for production or real security purposes.**

---

## Overview

`turb1600` follows the sponge construction model. The internal state is split into a rate and a capacity and processed in three phases:

1. Absorb  
2. Finalization  
3. Squeeze  

Input data is absorbed into the state in fixed-size blocks, permuted through multiple rounds, and output is extracted from the rate portion of the state.

No claims are made regarding cryptographic strength, collision resistance, or performance.

---

## Parameters

| Parameter | Value |
|---------|------|
| State size | 1600 bits |
| Word size | 64 bits |
| State words | 25 |
| Rate | 1088 bits (136 bytes) |
| Capacity | 512 bits |
| Output size | 1024 bits (128 bytes) |
| Rounds per block | 16 |
| Final rounds | 4 |

---

## State Initialization

The internal state is initialized by XOR-mixing a fixed seed string into the 1600-bit state:

```
turb1600 | sponge-hash | state=1600 | rate=1088 | capacity=512 | output=1024 | v1
```

This provides basic domain separation from other sponge constructions.

---

## Padding

The message is padded using a Keccak-style padding rule:

```
message || 0x01 || 0x00 ... || 0x80
```

Padding ensures the input length aligns with the sponge rate.

---

## Permutation Structure

Each permutation round consists of the following layers:

1. **Theta diffusion**  
   Column parity mixing across the 5×5 state

2. **Bit permutation and rotation**  
   Fixed rotation offsets per lane and lane relocation using `(i * 7) mod 25`

3. **Chi non-linearity**  
   Boolean nonlinear mixing within rows

4. **Round constant injection**  
   XOR into lane 0 using a generated round constant

A total of 1024 round constants are generated using a xorshift-style process seeded with:

```
0x9E3779B97F4A7C15
```

---

## Sponge Phases

### Absorb
- Input is processed in 136-byte blocks
- Each block is XORed into the state
- 16 permutation rounds are applied per block

### Finalization
- After all blocks are absorbed, 4 additional rounds are applied

### Squeeze
- Output is extracted from the rate portion of the state
- Permutations continue until 1024 bits are produced

---

## Reference Implementations

### Python
- Clear, readable reference implementation
- Suitable for understanding and verification
- Includes a built-in self-test

### Rust
- Fixed-size state representation
- Mirrors the Python implementation closely
- Exposes a reusable hash function

---

## Command-Line Interface

The Rust CLI allows hashing of different input types.

### Usage

```
turb1600 <string>
turb1600 --hex <hex-string>
turb1600 --file <path>
turb1600 --tag <tag> <string>
```

The `--tag` option performs domain-separated hashing by prepending:

```
<tag> || 0x00 || <message>
```

---

## Security Notes

- Not standardized
- Not audited
- No formal cryptanalysis
- No security guarantees
- Not suitable for cryptographic use

---

## License

License not yet specified.
