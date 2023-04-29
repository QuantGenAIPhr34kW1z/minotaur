# User Guide - MINOTAUR v2.0

## CSTNSystems Deterministic Reduced-Order Turbofan Cycle Solver

This guide provides comprehensive instructions for using MINOTAUR in research and engineering workflows.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Understanding the Solver](#understanding-the-solver)
4. [Common Workflows](#common-workflows)
5. [Troubleshooting](#troubleshooting)
6. [Best Practices](#best-practices)

---

## Installation

### Prerequisites

| Component | Version | Purpose |
|-----------|---------|---------|
| gfortran | 9+ | Fortran compiler |
| fpm | 0.8+ | Fortran Package Manager |
| Rust | 1.70+ | Orchestration layer |
| Python | 3.8+ | Python bindings (optional) |
| gnuplot | 5+ | Visualization (optional) |

### Build from Source

```bash
# Clone repository
git clone https://github.com/CSTNSystems/minotaur.git
cd minotaur

# Build Fortran + Rust
make build

# Verify installation
./target/release/minotaur version
```

### Python Package

```bash
cd python
pip install maturin
maturin develop

# Verify
python -c "import minotaur; print(minotaur.__version__)"
```

---

## Quick Start

### 1. Single-Point Calculation

Create a configuration file:

```toml
# my_config.toml
[CSTNSystems]
program = "CSTNSystems"
module = "minotaur"
version = "1.0.0"

[solver]
max_iter = 64
tol = 1.0e-10
damping = 0.5

[invariants]
mass_tol = 1.0e-9
energy_tol = 1.0e-9

[constraints]
t4_max = 1400.0

[cycle]
mach = 0.65
alt_km = 8.0
bpr = 0.6
opr = 8.0
eta_comp = 0.82
eta_turb = 0.86
eta_nozz = 0.95
fuel_k = 1.0
```

Run:

```bash
minotaur run --config my_config.toml --out results/my_run.csv
```

### 2. Parameter Sweep

Add sweep section:

```toml
[sweep]
bpr_min = 0.2
bpr_max = 1.2
bpr_n = 21
opr_min = 4.0
opr_max = 14.0
opr_n = 21
```

Run:

```bash
minotaur sweep --config my_config.toml --out results/sweep.csv --json
```

### 3. Python Interface

```python
import minotaur
import numpy as np

# Single point
result = minotaur.solve(mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0)
print(f"T4 = {result.t4:.1f} K")

# Sweep
bpr = np.linspace(0.2, 1.2, 21)
opr = np.linspace(4.0, 14.0, 21)
results = minotaur.sweep(bpr, opr)
```

---

## Understanding the Solver

### The Three Gates

Every solution must pass through three validation gates:

```
┌─────────────────────────────────────────────────────────────┐
│                        INPUT                                │
│  mach, alt_km, bpr, opr, efficiencies, constraints          │
└──────────────────────────┬──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  GATE 1: ADMISSIBILITY                      │
│  • BPR ∈ [0, 2]                                             │
│  • All values finite (no NaN/Inf)                           │
│  • T4 ≤ T4_max                                              │
│                                                             │
│  FAIL → status = NONPHYSICAL (5) or CONSTRAINT_VIOL (4)     │
└──────────────────────────┬──────────────────────────────────┘
                           │ PASS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  GATE 2: CONVERGENCE                        │
│  • Damped Newton iteration                                  │
│  • Armijo backtracking line search                          │
│  • ‖residual‖ < tolerance                                   │
│                                                             │
│  FAIL → status = MAXITER (1) or DIVERGED (2)                │
└──────────────────────────┬──────────────────────────────────┘
                           │ PASS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  GATE 3: INVARIANTS                         │
│  • Mass residual < mass_tol                                 │
│  • Energy residual < energy_tol                             │
│                                                             │
│  FAIL → status = INVARIANT_VIOL (3)                         │
└──────────────────────────┬──────────────────────────────────┘
                           │ PASS
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                     OUTPUT (status = 0)                     │
│  T4, TSFC, Thrust, residuals, iterations                    │
└─────────────────────────────────────────────────────────────┘
```

### Status Code Interpretation

| Code | Name | What Happened | What To Do |
|------|------|---------------|------------|
| 0 | OK | Solution valid | Use results |
| 1 | MAXITER | Too many iterations | Increase `max_iter`, adjust `damping` |
| 2 | DIVERGED | Solution blew up | Input may be infeasible |
| 3 | INVARIANT_VIOL | Conservation violated | Check tolerances |
| 4 | CONSTRAINT_VIOL | T4 exceeded limit | Lower OPR or fuel_k |
| 5 | NONPHYSICAL | NaN or Inf detected | Check parameter bounds |

### Solver Parameters

| Parameter | Effect | Typical Range |
|-----------|--------|---------------|
| `damping` | Lower = more stable, slower | 0.3–0.8 |
| `max_iter` | Iteration budget | 50–500 |
| `tol` | Convergence criterion | 1e-8–1e-12 |
| `mass_tol` | Mass conservation gate | 1e-8–1e-10 |
| `energy_tol` | Energy conservation gate | 1e-8–1e-10 |

---

## Common Workflows

### 1. Design Space Exploration

```bash
# Generate 441-point sweep
minotaur sweep --config configs/sweep.toml --out results/sweep.csv --json

# Visualize
gnuplot -e "datafile='results/sweep.csv'" plots/plot_fuel_vs_bpr.gp
```

### 2. Sensitivity Analysis

```bash
# Finite differences (approximate)
minotaur sensitivity --config baseline.toml --step 1e-6

# Automatic differentiation (exact)
minotaur jacobian --config baseline.toml --json
```

### 3. Degradation Study

```bash
# Compare nominal vs degraded
minotaur compare --config baseline.toml --level moderate --json

# All three levels
for level in light moderate severe; do
    minotaur compare --config baseline.toml --level $level --out results/deg_$level.csv
done
```

### 4. Multi-Objective Optimization

```bash
# Find Pareto-optimal designs
minotaur optimize \
    --config baseline.toml \
    --pop-size 100 \
    --generations 50 \
    --seed 42 \
    --json

# Visualize Pareto front
gnuplot -e "datafile='results/pareto_front.csv'" plots/pareto_front.gp
```

### 5. Python Workflow

```python
import minotaur
import numpy as np
import matplotlib.pyplot as plt

# Sweep and visualize
bpr = np.linspace(0.2, 1.2, 31)
opr = np.linspace(4.0, 14.0, 31)
results = minotaur.sweep(bpr, opr)

# Reshape for plotting
n_bpr, n_opr = results['n_bpr'], results['n_opr']
TSFC = results['tsfc'].reshape(n_opr, n_bpr)
BPR = results['bpr'].reshape(n_opr, n_bpr)
OPR = results['opr'].reshape(n_opr, n_bpr)

# Contour plot
plt.contourf(BPR, OPR, TSFC, levels=20)
plt.colorbar(label='TSFC')
plt.xlabel('BPR')
plt.ylabel('OPR')
plt.savefig('tsfc_contour.png')
```

---

## Troubleshooting

### Problem: Solver doesn't converge (MAXITER)

**Symptoms:** status = 1, many iterations

**Solutions:**
1. Increase `max_iter`:
   ```toml
   [solver]
   max_iter = 200
   ```

2. Reduce damping (slower but more stable):
   ```toml
   [solver]
   damping = 0.3
   ```

3. Check if operating point is feasible

### Problem: Divergence (DIVERGED)

**Symptoms:** status = 2, solver exits early

**Solutions:**
1. Check parameter bounds:
   - Is OPR too high for the BPR?
   - Is Mach number feasible?

2. Start from a known-good configuration and perturb gradually

### Problem: Constraint violation (CONSTRAINT_VIOL)

**Symptoms:** status = 4, T4 exceeds limit

**Solutions:**
1. Increase T4_max if physically justified:
   ```toml
   [constraints]
   t4_max = 1600.0
   ```

2. Reduce OPR or fuel_k

### Problem: NaN in output (NONPHYSICAL)

**Symptoms:** status = 5

**Solutions:**
1. Check for zero or negative values where not allowed
2. Verify efficiency values are in [0, 1]
3. Check altitude is reasonable

### Problem: Large invariant residuals

**Symptoms:** status = 3

**Solutions:**
1. Tighten solver tolerance:
   ```toml
   [solver]
   tol = 1.0e-12
   ```

2. Loosen invariant thresholds (use with caution):
   ```toml
   [invariants]
   mass_tol = 1.0e-8
   ```

---

## Best Practices

### 1. Start Simple

Begin with the baseline configuration and modify one parameter at a time:

```bash
# Verify baseline works
minotaur run --config configs/baseline.toml

# Then modify
minotaur run --config my_modified.toml
```

### 2. Validate Before Sweeping

Before running large sweeps:

```bash
minotaur validate --config my_sweep.toml
```

### 3. Use JSON for Post-Processing

Always generate JSON manifests for important runs:

```bash
minotaur run --config config.toml --json
```

The manifest captures:
- Exact configuration
- Solver version
- Platform info
- Timestamps

### 4. Reproducibility

For reproducible results:

1. Record exact compiler versions
2. Use the same platform
3. Avoid fast-math flags
4. Save JSON manifests

```bash
# Record environment
gfortran --version > env.txt
rustc --version >> env.txt
uname -a >> env.txt
```

### 5. Parameter Bounds

Stay within validated parameter ranges:

| Parameter | Valid Range | Notes |
|-----------|-------------|-------|
| mach | [0, 0.95] | Subsonic only |
| alt_km | [0, 20] | Standard atmosphere |
| bpr | [0.1, 2.0] | Low bypass |
| opr | [2, 20] | Compact cores |
| eta_comp | [0.7, 0.95] | Realistic efficiency |
| eta_turb | [0.8, 0.95] | Realistic efficiency |

### 6. Interpreting Results

**TSFC proxy:** Lower is better (less fuel per unit thrust)

**Thrust proxy:** Higher is better

**T4:** Must stay below T4_max for material limits

**Iterations:** Higher counts may indicate marginal feasibility

---

## Example Configurations

### High-Efficiency Design

```toml
[cycle]
mach = 0.65
alt_km = 10.0
bpr = 0.8
opr = 10.0
eta_comp = 0.88
eta_turb = 0.90
eta_nozz = 0.97
```

### Aggressive High-Thrust

```toml
[cycle]
mach = 0.80
alt_km = 6.0
bpr = 0.4
opr = 12.0

[constraints]
t4_max = 1500.0  # Higher thermal limit
```

### Degraded Engine Study

```toml
[cycle]
mach = 0.65
alt_km = 8.0
bpr = 0.6
opr = 8.0

[degradation]
eta_comp_factor = 0.92  # 8% compressor degradation
eta_turb_factor = 0.95  # 5% turbine degradation
loss_adder = 0.015      # Additional pressure losses
```

---

*CSTNSystems - Compact Subsonic Turbofan Numerical Systems*
