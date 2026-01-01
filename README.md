# turb1600

`turb1600` is an **experimental sponge-based cryptographic hash function** with a  
**1600-bit internal state** and a **1024-bit output**.

⚠️ **Status: Experimental**  
This project is under active development. The algorithm, parameters, and public
interfaces may change. **Do not use in production or security-critical systems.**

---

## Features

- 1600-bit internal state (25 × 64-bit lanes)
- Sponge construction (1088-bit rate, 512-bit capacity)
- Deterministic and reproducible
- Reference-first design
- Bit-for-bit compatible with Python reference


---

## Python Installation (via maturin)

### Requirements
- Rust (stable)
- Python 3.9+
- `maturin`

Install maturin:
```bash
pip install maturin
cd rust
maturin develop --release
```

or Build a wheel:

```bash
maturin build --release
pip install target/wheels/turb1600*.whl
```

# Python Usage
```bash
import turb1600
digest = turb1600.hash(b"hello world")
```

