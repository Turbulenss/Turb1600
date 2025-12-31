# turb1600

`turb1600` is an experimental sponge-based cryptographic hash function with a
1600-bit internal state and a 1024-bit output.

This repository currently provides:
- A **reference implementation in Rust**
- **Bit-for-bit compatibility** with the Python reference implementation

⚠️ **Project status:**  
This project is under active development. The algorithm, parameters, and public
interfaces may change. Do **not** use this implementation for production or
security-critical purposes at this stage.

---

## Overview

`turb1600` follows a sponge construction inspired by Keccak-style permutations,
using:
- 1600-bit internal state
- 1088-bit rate
- 512-bit capacity
- 64-bit lanes
- Strong θ-style diffusion and nonlinear mixing

The design prioritizes clarity and determinism for experimentation and analysis.

---

Full documentation for `turb1600` comming soon