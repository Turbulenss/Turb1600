# turb1600 (Python Wrapper)

`turb1600` is an **experimental sponge-based cryptographic hash function** exposed to Python via **PyO3**.

⚠️ **Status: Experimental**
This wrapper is for **testing, research, and experimentation only**. Do **not** use in production or security-critical systems.

---

## Overview

* 1600-bit internal state (25 × 64-bit lanes)
* 1024-bit (128-byte) output
* Sponge construction (1088-bit rate, 512-bit capacity)
* Deterministic and reproducible
* Rust core with Python bindings

---

## Requirements

* Python **3.9+**
* Rust **stable** toolchain
* `maturin`

Install maturin:

```bash
pip install maturin
```

---

## Build & Install (Local)

From the `rust/` directory:

```bash
maturin develop --release
```

Or build a wheel:

```bash
maturin build --release
pip install target/wheels/turb1600*.whl
```

---

## Python Usage

```python
import turb1600

digest = turb1600.hash(b"hello world")
print(len(digest))  # 128 bytes
```

The function returns raw bytes. Encode as hex if needed:

```python
import binascii
print(binascii.hexlify(digest).decode())
```

---

## API

### `turb1600.hash(data: bytes) -> bytes`

* **Input:** arbitrary byte string
* **Output:** 128-byte digest

---

## Notes

* Algorithm parameters may change
* No stability or security guarantees
* Intended to stay bit-compatible with the Rust reference

---

## License

MIT
