# MINOTAUR (CSTNSystems) v2.9.0

## Deterministic Reduced-Order Numerical Modeling of Compact Subsonic Turbofan Cycles

---

### Abstract

MINOTAUR is a deterministic numerical framework for reduced-order modeling of compact low-bypass turbofan cycles operating in constrained subsonic regimes. The framework emphasizes physical admissibility, conservation-law invariants, and bitwise reproducibility.

The implementation couples a **Fortran 2008** numerical core with a **Rust 2021** orchestration layer, providing:

- Damped Newton solving with Armijo backtracking
- Hard constraint enforcement (thermal ceiling T₄ ≤ T₄,max)
- Post-solve invariant verification (mass/energy conservation)
- Deterministic parameter sweeps and sensitivity analysis
- Modular component models with degradation analysis
- Stable schema versioning with CI/CD integration
- Python bindings via PyO3 with NumPy integration
- Forward-mode automatic differentiation for exact gradients
- NSGA-II multi-objective optimization
- Comprehensive API documentation and test coverage
- Variable specific heats, real gas effects, combustion models
- JSON manifests for reproducible research workflows

---

### Research Program Identity

This software is part of:

**Compact Subsonic Turbofan Numerical Systems (CSTNSystems)**
A modular numerical framework for compact propulsion systems operating in constrained subsonic regimes.

---

### Current Version: v2.9.0

| Milestone | Status      | Description                                          |
| --------- | ----------- | ---------------------------------------------------- |
| v2.1      | ✅ Complete | Runnable kernel, FFI boundary, result bundles        |
| v2.2      | ✅ Complete | Convergent solver, invariant gates, baseline configs |
| v2.3      | ✅ Complete | Sweeps, sensitivity analysis, reproducibility docs   |
| v2.4      | ✅ Complete | Model modularity, degradation scenarios              |
| v2.5      | ✅ Complete | Schema versioning, long-term stability               |
| v2.6      | ✅ Complete | Python bindings via PyO3, NumPy integration          |
| v2.8      | ✅ Complete | Forward-mode automatic differentiation               |
| v2.9      | ✅ Complete | NSGA-II multi-objective optimization                 |
| v2.9      | ✅ Complete | Production release, comprehensive docs & tests       |
| v2.9      | ✅ Complete | Extended physics: Cp(T), real gas, combustion        |

---

### New Features by Version

| Version | Feature                   | Description                                            |
| ------- | ------------------------- | ------------------------------------------------------ |
| v2.4    | Component Models          | Standard/advanced compressor, turbine, nozzle models   |
| v2.4    | Loss Coefficients         | Configurable inlet, burner, turbine, nozzle losses     |
| v2.4    | Degradation Analysis      | `minotaur compare` with light/moderate/severe levels |
| v2.5    | Schema v2.0.0             | Stable JSON output format with versioning              |
| v2.5    | Version Command           | `minotaur version` shows full version info           |
| v2.5    | CI/CD                     | GitHub Actions for Linux, macOS, determinism checks    |
| v2.6    | Python Bindings           | PyO3-based package with NumPy integration              |
| v2.6    | Jupyter Examples          | Interactive notebooks for tutorials                    |
| v2.8    | Automatic Differentiation | Forward-mode AD via dual numbers                       |
| v2.8    | Exact Jacobian            | 6×3 Jacobian without truncation error                 |
| v2.9    | NSGA-II                   | Multi-objective optimization algorithm                 |
| v2.9    | Pareto Fronts             | Bi-objective trade-off visualization                   |
| v2.9    | API Documentation         | Complete reference for all interfaces                  |
| v2.9    | User Guide                | Workflows, troubleshooting, best practices             |
| v2.9    | Test Suite                | Property-based tests, regression tests                 |
| v2.9    | Variable Cp               | NASA polynomial Cp(T) for air and products             |
| v2.9    | Real Gas                  | Peng-Robinson EOS compressibility factor               |
| v2.9    | Combustion Models         | Equilibrium and kinetic combustion                     |

---

### Key Features

#### Numerical Solver

- **Damped Newton iteration** with configurable damping factor
- **Armijo backtracking line search** for globalized convergence
- **Divergence detection** (10× residual increase triggers termination)
- **Admissibility projection** (β ∈ [0, 2])

#### Physical Constraints

- **Thermal ceiling enforcement** (T₄ ≤ T₄,max)
- **Mass conservation residual** verification
- **Energy conservation residual** verification
- **6-class failure taxonomy** for structured diagnostics

#### Component Models

- **Standard models**: Basic isentropic efficiency formulations
- **Advanced models**: Polytropic effects, cooling, divergence losses
- **Configurable losses**: Inlet, burner, turbine, nozzle coefficients

#### Degradation Analysis

- **Light**: 5% efficiency loss, +1% pressure loss
- **Moderate**: 10% efficiency loss, +2% pressure loss
- **Severe**: 15% efficiency loss, +3% pressure loss

#### Research Outputs

- **Parameter sweeps** across (β, π) grid
- **Sensitivity analysis** via central finite differences
- **JSON manifests** with git commit, platform, config hash
- **Deterministic execution** (bitwise reproducible)

---

### Quick Start

```bash
# Build
cd src/fortran && fpm build --profile release
cd ../rust && cargo build --release

# Single run
./target/release/minotaur run \
    --config configs/baseline.toml \
    --out results/out.csv \
    --json

# Parameter sweep (441 points)
./target/release/minotaur sweep \
    --config configs/sweep.toml \
    --out results/sweep.csv \
    --json

# Sensitivity analysis
./target/release/minotaur sensitivity \
    --config configs/baseline.toml \
    --step 1e-6

# Degradation comparison (v2.4)
./target/release/minotaur compare \
    --config configs/baseline.toml \
    --level moderate \
    --json

# Version information
./target/release/minotaur version

# Validate configuration
./target/release/minotaur validate --config configs/baseline.toml
```

---

### Status Code Taxonomy

| Code | Symbol          | Description                                         |
| ---- | --------------- | --------------------------------------------------- |
| 0    | OK              | Converged, all constraints and invariants satisfied |
| 1    | MAXITER         | Iteration limit reached without convergence         |
| 2    | DIVERGED        | Residual norm increased 10× from initial value     |
| 3    | INVARIANT_VIOL  | Mass or energy conservation violated                |
| 4    | CONSTRAINT_VIOL | Thermal ceiling T₄,max exceeded                    |
| 5    | NONPHYSICAL     | Non-finite values (NaN, ±Inf) detected             |

---

### Performance Metrics (v2.3 Sweep Results)

| Metric             | Value                 |
| ------------------ | --------------------- |
| Grid size          | 21 × 21 = 441 points |
| Convergence rate   | 94.7% (418/441)       |
| Thermal violations | 5.2% (23/441)         |
| Mean iterations    | 14.3                  |
| Iteration range    | 8–47                 |

---

### Degradation Impact (v2.4 Results)

| Degradation Level | TSFC Change | Thrust Change | T₄ Change |
| ----------------- | ----------- | ------------- | ---------- |
| Light (5%)        | +5.9%       | -3.1%         | +28 K      |
| Moderate (10%)    | +11.1%      | -6.0%         | +57 K      |
| Severe (15%)      | +17.3%      | -9.2%         | +89 K      |

---

### Sensitivity Jacobian (at β=0.6, π=8)

| Parameter | ∂TSFC/∂p | ∂Thrust/∂p | ∂T₄/∂p |
| --------- | ---------- | ------------ | --------- |
| BPR (β)  | −0.300    | −0.172      | 0.000     |
| OPR (π)  | +0.020     | +0.148       | +56.7     |
| ηcomp    | −0.112    | +0.028       | −72.4    |
| ηturb    | −0.021    | +0.012       | 0.000     |
| Mach      | +0.023     | +0.156       | +12.8     |
| Altitude  | −0.002    | −0.021      | −1.71    |

---

### Documentation

| Document                    | Description                                        |
| --------------------------- | -------------------------------------------------- |
| `docs/REPRODUCIBILITY.md` | Build commands, manifest format, determinism notes |
| `docs/API_REFERENCE.md`   | Complete API reference (v2.0)                      |
| `docs/USER_GUIDE.md`      | Workflows, troubleshooting, best practices         |
|                             |                                                    |
| `ROADMAP.md`              | Development milestones and work breakdown          |
| `IMPROVEMENTS.md`         | Planned enhancements and status tracking           |

---

### Python Bindings (v2.6)

```python
import minotaur
import numpy as np

# Single-point solution
result = minotaur.solve(mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0)
print(f"T4 = {result.t4:.1f} K, TSFC = {result.tsfc_proxy:.4f}")

# Parameter sweep with NumPy
bpr = np.linspace(0.2, 1.2, 21)
opr = np.linspace(4.0, 14.0, 21)
results = minotaur.sweep(bpr, opr)
```

---

### Automatic Differentiation (v2.8)

Exact gradients via forward-mode AD (no truncation error):

```bash
minotaur jacobian --config baseline.toml --json
```

Output: 6×3 Jacobian matrix ∂(TSFC, Thrust, T₄)/∂(M, h, β, π, ηc, ηt)

---

### Multi-objective Optimization (v2.9)

NSGA-II algorithm for Pareto-optimal designs:

```bash
minotaur optimize --config baseline.toml --pop-size 100 --generations 50 --json
```

Bi-objective: minimize TSFC, maximize Thrust with T₄ constraint.

---

### Extended Physics (v2.9)

#### Variable Specific Heats

- NASA polynomial fits for Cp(T)
- Air and combustion products
- Temperature range: 200-6000 K

#### Real Gas Effects

- Peng-Robinson equation of state
- Compressibility factor Z(T, P)
- Accurate at high pressures

#### Combustion Models

| Model       | Description                        |
| ----------- | ---------------------------------- |
| Simple      | Constant Cp, fixed efficiency      |
| Equilibrium | Adiabatic flame via energy balance |
| Kinetic     | Finite-rate Arrhenius chemistry    |

---

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Rust Orchestration                    │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │
│  │   CLI   │  │ Config  │  │  Sweep  │  │   I/O   │    │
│  │  clap   │  │  toml   │  │  Grid   │  │ CSV/JSON│    │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘    │
│       └────────────┴────────────┴────────────┘          │
│                         │ FFI (C ABI)                   │
├─────────────────────────┼───────────────────────────────┤
│                         ▼                               │
│                Fortran Numerical Core                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Solver    │  │ Invariants  │  │  Components │     │
│  │   Newton    │  │ Mass/Energy │  │  Comp/Turb  │     │
│  │ + Armijo    │  │  Residuals  │  │   /Nozzle   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

---

### Intended Use

This software is intended for:

- Academic and pre-design numerical experimentation
- Feasibility corridor mapping
- Sensitivity and trade-off studies
- Degradation impact assessment
- Solver robustness evaluation
- Reproducible research workflows

---

### Requirements

| Component | Version |
| --------- | ------- |
| gfortran  | 11.4+   |
| fpm       | 0.9+    |
| rustc     | 1.75+   |
| cargo     | 1.75+   |

---

### License

See LICENSE file for details.

---

### Citation

```bibtex
@software{minotaur,
  title  = {MINOTAUR v2.9.0},
  author = {CSTNSystems},
  year   = {2023},
  url    = {https://github.com/CSTNSystems/minotaur},
  note   = {Compact Subsonic Turbofan Numerical Systems}
}
```

---

*CSTNSystems - Compact Subsonic Turbofan Numerical Systems*
