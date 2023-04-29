# Reproducibility Guide - MINOTAUR v2.3

## 1. Exact Build Commands

### Prerequisites

```bash
# Fortran compiler
gfortran --version  # Tested: GCC 11+

# Fortran Package Manager
pip install fpm     # or: cargo install fpm
fpm --version       # Tested: 0.9+

# Rust toolchain
rustc --version     # Tested: 1.70+
cargo --version
```

### Build Sequence

```bash
# Clone repository
git clone https://github.com/CSTNSystems/minotaur.git
cd minotaur

# Build Fortran library
cd src/fortran
fpm build --profile release

# Build Rust orchestration (includes Fortran linkage)
cd ../rust
cargo build --release

# Verify installation
./target/release/minotaur --version
```

### Single Command Build

```bash
make build  # Builds both Fortran and Rust components
```

## 2. Manifest Format

Every run produces a JSON manifest containing reproducibility metadata:

```json
{
  "manifest": {
    "schema_version": "0.1.0",
    "solver_version": "0.3.0",
    "timestamp_utc": "2023-01-18T12:00:00Z",
    "git_commit": "abc1234",
    "git_dirty": false,
    "platform": "linux",
    "rust_version": "stable",
    "config_hash": "sha256:..."
  }
}
```

### Generating Manifests

```bash
# Single run with JSON output
minotaur run --config configs/baseline.toml --out results/out.csv --json

# Sweep with JSON summary
minotaur sweep --config configs/sweep.toml --out results/sweep.csv --json
```

## 3. Determinism Contract

Given identical:
- Configuration file (byte-for-byte)
- Compiler versions (gfortran, rustc)
- Platform (OS, CPU architecture)

MINOTAUR produces **bitwise identical** results.

### Verification Procedure

```bash
# Run twice
minotaur run --config configs/baseline.toml --out run1.csv --json
minotaur run --config configs/baseline.toml --out run2.csv --json

# Compare outputs
diff run1.csv run2.csv        # Should be empty
diff run1.json run2.json      # Timestamps differ; values identical
```

## 4. Known Sources of Non-Determinism

### 4.1 Floating-Point Mode

| Flag | Effect | Recommendation |
|------|--------|----------------|
| `-ffast-math` | Reorders FP operations | **Never use** |
| `-ffp-contract=fast` | Fuses multiply-add | Avoid for reproducibility |
| `-march=native` | CPU-specific optimizations | Document target |

**Default (safe) compilation:**
```bash
gfortran -O2 -fno-fast-math ...
```

### 4.2 Compiler Version Differences

- GCC 11 vs GCC 12: May produce different results at `-O3`
- Recommendation: Pin compiler version in CI

### 4.3 Platform Differences

| Platform | Notes |
|----------|-------|
| x86_64 Linux | Reference platform |
| x86_64 macOS | Generally matches Linux |
| ARM64 | May differ due to FMA instructions |

### 4.4 Library Versions

- glibc math functions may vary
- BLAS/LAPACK not used (no external deps)

## 5. Baseline Verification

### Running All Baselines

```bash
# Test all 10 baseline configurations
for cfg in tests/baselines/config_*.toml; do
  echo "Testing: $cfg"
  minotaur run --config "$cfg" --out /tmp/test.csv
done
```

### Golden Hash Verification

```bash
# Generate hash of results
sha256sum results/out_baseline.csv

# Compare with stored baseline
diff <(sha256sum results/out_baseline.csv) tests/baselines/golden_hashes.txt
```

## 6. Environment Recording

### Capture System State

```bash
# Record environment for reproducibility
cat > results/environment.txt << EOF
Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Hostname: $(hostname)
OS: $(uname -a)
gfortran: $(gfortran --version | head -1)
rustc: $(rustc --version)
cargo: $(cargo --version)
fpm: $(fpm --version 2>/dev/null || echo "not installed")
Git commit: $(git rev-parse HEAD 2>/dev/null || echo "not a git repo")
Git dirty: $(git diff --quiet 2>/dev/null && echo "no" || echo "yes")
EOF
```

## 7. Result Schema

### CSV Columns

| Column | Type | Description |
|--------|------|-------------|
| case | string | Run identifier |
| bpr | float | Bypass ratio |
| opr | float | Overall pressure ratio |
| mach | float | Flight Mach number |
| alt_km | float | Altitude [km] |
| status | int | Solver exit code (0=OK) |
| converged | bool | status==0 |
| iter | int | Iteration count |
| mass_resid | float | Mass conservation residual |
| energy_resid | float | Energy conservation residual |
| final_residual | float | Final solver residual |
| final_bpr | float | Converged BPR value |
| t4 | float | Thermal proxy |
| tsfc_proxy | float | TSFC proxy |
| thrust_proxy | float | Thrust proxy |

### JSON Schema

See `results/schema.json` for formal JSON Schema definition.

## 8. Troubleshooting

### Non-Matching Results

1. **Check compiler versions** - Must match exactly
2. **Check FP flags** - No fast-math
3. **Check config file** - Byte-for-byte identical
4. **Check platform** - Same architecture

### Solver Failures

| Status | Meaning | Action |
|--------|---------|--------|
| 1 | MAXITER | Increase max_iter or adjust damping |
| 2 | DIVERGED | Reduce damping, check initial guess |
| 3 | INVARIANT_VIOL | Check tolerances |
| 4 | CONSTRAINT_VIOL | T4 exceeded limit |
| 5 | NONPHYSICAL | Check input parameters |

## 9. Citation

When publishing results obtained with MINOTAUR, include:

1. Solver version
2. Configuration file (or hash)
3. Platform/compiler information
4. Git commit (if from source)

```bibtex
@software{minotaur,
  title  = {MINOTAUR v2.3.0},
  author = {CSTNSystems},
  year   = {2023},
  url    = {https://github.com/CSTNSystems/minotaur},
  note   = {Commit: abc1234, Platform: x86_64-linux-gnu, GCC 11.4}
}
```
