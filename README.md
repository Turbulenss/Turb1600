# turb1600 — High-Performance Sponge-Based Hash Engine

---

## Overview

`turb1600` is a fast, modern, sponge-based cryptographic hash function with a 1600-bit internal state. It delivers a 1024-bit output and is designed with strong diffusion, high throughput, and robust domain separation.

Key features:

* **Internal State:** 1600-bit (25×64-bit lanes)
* **Rate:** 1088-bit, **Capacity:** 512-bit
* **Output:** 1024-bit
* **Rounds:** 36 main rounds + 6 final rounds
* **Domain Separation:** Built-in unique seed tag
* **Portable & Deterministic:** Same output across platforms

`turb1600` is suitable for hashing arbitrary-length inputs, including strings, files, or binary data.

---

## Installation

Add `turb1600` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
turb1600 = "0.2"
hex = "0.4"  # optional for hex encoding
```

Then include it in your code:

```rust
use turb1600::turb1600_hash;
```

---

## Usage

### Hashing a string

```rust
let data = b"hello world";
let digest = turb1600_hash(data);
println!("{:02x?}", digest);
```

### Hashing arbitrary bytes

```rust
let bytes = [0u8, 1, 2, 3, 4];
let digest = turb1600_hash(&bytes);
println!("{:02x?}", digest);
```

### Convenience: Hex output

```rust
pub fn hash_hex(data: &str) -> String {
    let digest = turb1600_hash(data.as_bytes());
    digest.iter().map(|b| format!("{:02x}", b)).collect()
}

let hex = hash_hex("example");
println!("{}", hex);
```

---

## Command-Line Interface (CLI)

`turb1600` provides a flexible CLI:

```text
Usage:
  turb1600 <string>                 Hash a string
  turb1600 --hex <hex-string>       Hash raw bytes from hex
  turb1600 --file <path>            Hash file contents
  turb1600 --tag <tag> <string>     Hash string with domain tag
Options:
  --raw                              Output raw bytes instead of hex
```

Examples:

```bash
turb1600 "hello world"
turb1600 --hex 616263
turb1600 --file ./myfile.txt
turb1600 --tag mytag "message"
```

---

## Design Highlights

* **Sponge Construction:** Absorb and squeeze phases allow for arbitrary-length input and fixed-length output.
* **Column Mixing (Theta):** Ensures diffusion across all lanes.
* **Rotation + Permutation (Rho + Pi):** Adds position-dependent rotations and permutations for avalanche effect.
* **Nonlinear Layer (Chi):** Introduces nonlinearity to prevent simple algebraic attacks.
* **Round Constants (Iota):** Unique constants for each round ensure domain separation and prevent symmetry.

The algorithm guarantees deterministic, uniform hashing with strong diffusion properties.

---

## Testing

Run unit tests to verify correctness:

```bash
cargo test
```

Expected behavior:

* Hashing known inputs produces 128-byte (1024-bit) outputs.
* Hex conversion produces 256-character strings.

Example tests:

```rust
#[test]
fn test_basic_hash() {
    let msg = b"hello world";
    let digest = turb1600_hash(msg);
    assert_eq!(digest.len(), 128);
}

#[test]
fn test_hash_hex() {
    let hex = hash_hex("test");
    assert_eq!(hex.len(), 256);
}
```

---

## Performance

`turb1600` is optimized for Rust:

* Efficient memory layout for 64-bit architectures
* Minimal allocations
* Inline rotations and bit operations

Benchmarks show competitive throughput for long messages, making it suitable for high-performance applications.

---

## Security Considerations

`turb1600` is designed for strong diffusion, avalanche effect, and deterministic outputs. While it is robust for experimental and educational purposes, ensure it meets your application security requirements before deployment in production environments.

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Project Structure

```text
.
├── LICENSE
├── README.md
├── ref/           # Reference Python implementation for clarity
│   └── turb1600.py
└── rust/
    ├── Cargo.toml
    └── src/
        ├── core.rs  # Core hashing engine
        ├── lib.rs   # Public API
        └── main.rs  # CLI entry point
```

---

`turb1600` is a clean, deterministic, and efficient Rust-based hash engine, designed for clarity, portability, and high-performance cryptographic applications.
