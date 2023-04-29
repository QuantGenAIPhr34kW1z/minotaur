# API Reference - MINOTAUR v2.9

## CSTNSystems Deterministic Reduced-Order Turbofan Cycle Solver

This document provides comprehensive API documentation for all MINOTAUR interfaces.

---

## Table of Contents

1. [CLI Interface](#cli-interface)
2. [Rust API](#rust-api)
3. [Python API](#python-api)
4. [Fortran API](#fortran-api)
5. [Configuration Reference](#configuration-reference)
6. [Status Codes](#status-codes)
7. [Output Formats](#output-formats)

---

## CLI Interface

### Global Options

```
minotaur [OPTIONS] <COMMAND>

Options:
  -c, --config <PATH>    Path to TOML configuration file
  -o, --out <PATH>       Output path (file or directory)
  -m, --mode <MODE>      Run mode: "single" or "sweep" [default: single]
  -h, --help             Print help
  -V, --version          Print version
```

### Commands

#### `run` - Single Configuration

```bash
minotaur run --config <PATH> [--out <PATH>] [--json]
```

Executes a single cycle calculation from the provided configuration.

| Option       | Description                                         |
| ------------ | --------------------------------------------------- |
| `--config` | Path to TOML configuration file (required)          |
| `--out`    | Output CSV path [default: results/out_baseline.csv] |
| `--json`   | Generate JSON manifest alongside CSV                |

**Example:**

```bash
minotaur run --config configs/baseline.toml --out results/run.csv --json
```

#### `sweep` - Parameter Sweep

```bash
minotaur sweep --config <PATH> [--out <PATH>] [--json]
```

Executes a grid sweep over BPR and OPR ranges defined in `[sweep]` section.

**Example:**

```bash
minotaur sweep --config configs/sweep.toml --out results/sweep.csv --json
```

#### `sensitivity` - Sensitivity Analysis

```bash
minotaur sensitivity --config <PATH> [--out <PATH>] [--step <FLOAT>]
```

Computes local sensitivities via central finite differences.

| Option     | Description                                               |
| ---------- | --------------------------------------------------------- |
| `--step` | Relative step size for finite differences [default: 1e-6] |

**Output:** 6×4 Jacobian matrix (6 parameters × 4 outputs)

#### `jacobian` - Exact Jacobian via AD

```bash
minotaur jacobian --config <PATH> [--out <PATH>] [--json]
```

Computes exact Jacobian using forward-mode automatic differentiation.

**Output:** 6×3 Jacobian matrix with machine precision

#### `compare` - Degradation Comparison

```bash
minotaur compare --config <PATH> --level <LEVEL> [--out <PATH>] [--json]
```

Compares nominal vs degraded performance.

| Level        | η_comp Factor | η_turb Factor | Loss Adder  |
| ------------ | -------------- | -------------- | ----------- |
| `light`    | 0.95           | 0.97           | +0.01       |
| `moderate` | 0.90           | 0.94           | +0.02       |
| `severe`   | 0.85           | 0.91           | +0.03       |
| `custom`   | From config    | From config    | From config |

#### `optimize` - Multi-Objective Optimization

```bash
minotaur optimize --config <PATH> [OPTIONS]
```

| Option            | Description                     | Default |
| ----------------- | ------------------------------- | ------- |
| `--pop-size`    | Population size                 | 100     |
| `--generations` | Number of generations           | 50      |
| `--seed`        | Random seed for reproducibility | 42      |
| `--json`        | Generate JSON output            | false   |

**Objectives:** Minimize TSFC, Maximize Thrust

#### `validate` - Configuration Validation

```bash
minotaur validate --config <PATH>
```

Validates configuration file syntax and parameter bounds.

#### `version` - Version Information

```bash
minotaur version
```

Displays solver version, schema version, and feature summary.

---

## Rust API

### Core Types

#### `MinotaurInput`

```rust
#[repr(C)]
pub struct MinotaurInput {
    pub mach: f64,        // Flight Mach number [0, 0.95]
    pub alt_km: f64,      // Altitude [km] [0, 20]
    pub bpr: f64,         // Bypass ratio
    pub opr: f64,         // Overall pressure ratio
    pub eta_comp: f64,    // Compressor efficiency [0.7, 0.95]
    pub eta_turb: f64,    // Turbine efficiency [0.8, 0.95]
    pub eta_nozz: f64,    // Nozzle efficiency [0.9, 1.0]
    pub fuel_k: f64,      // Fuel parameter
    pub max_iter: i32,    // Maximum iterations [1, 10000]
    pub tol: f64,         // Convergence tolerance
    pub damping: f64,     // Newton damping factor (0, 1]
    pub mass_tol: f64,    // Mass conservation tolerance
    pub energy_tol: f64,  // Energy conservation tolerance
    pub t4_max: f64,      // Maximum T4 [K]
}
```

#### `MinotaurInputExt`

Extended input with component models and degradation:

```rust
pub struct MinotaurInputExt {
    // ... base fields from MinotaurInput ...

    // Component model selection
    pub compressor_model: i32,  // 0=standard, 1=advanced
    pub turbine_model: i32,
    pub nozzle_model: i32,

    // Loss coefficients
    pub inlet_loss: f64,
    pub burner_loss: f64,
    pub turbine_mech_loss: f64,
    pub nozzle_loss: f64,

    // Degradation factors
    pub eta_comp_factor: f64,   // Efficiency multiplier
    pub eta_turb_factor: f64,
    pub loss_adder: f64,
    pub is_degraded: i32,
}
```

#### `MinotaurOutput`

```rust
#[repr(C)]
pub struct MinotaurOutput {
    pub status: i32,          // Status code (0=OK)
    pub iter: i32,            // Iterations to convergence
    pub mass_resid: f64,      // Final mass residual
    pub energy_resid: f64,    // Final energy residual
    pub t4: f64,              // Turbine inlet temperature [K]
    pub tsfc_proxy: f64,      // TSFC proxy (non-dimensional)
    pub thrust_proxy: f64,    // Thrust proxy (non-dimensional)
    pub final_bpr: f64,       // Converged BPR
    pub final_residual: f64,  // Final residual norm
}
```

### Functions

#### `solve(inp: MinotaurInput) -> MinotaurOutput`

Solves a single cycle operating point.

```rust
use minotaur::ffi::{MinotaurInput, solve};

let inp = MinotaurInput {
    mach: 0.65,
    alt_km: 8.0,
    bpr: 0.6,
    opr: 8.0,
    // ... other fields ...
};

let out = solve(inp);
if out.status == 0 {
    println!("T4 = {} K, TSFC = {}", out.t4, out.tsfc_proxy);
}
```

#### `solve_ext(inp: MinotaurInputExt) -> MinotaurOutput`

Solves with extended component models and degradation.

#### `compute_jacobian(...) -> JacobianResult`

Computes exact 6×3 Jacobian via forward-mode AD.

```rust
let result = compute_jacobian(mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max);
// result.jacobian[i][j] = ∂output_j / ∂param_i
```

---

## Python API

### Installation

```bash
cd python
pip install maturin
maturin develop
```

### Module: `minotaur`

#### `solve(...)` → `SolverResult`

```python
import minotaur

result = minotaur.solve(
    mach=0.65,           # Flight Mach number
    alt_km=8.0,          # Altitude [km]
    bpr=0.6,             # Bypass ratio
    opr=8.0,             # Overall pressure ratio
    eta_comp=0.82,       # Compressor efficiency (optional)
    eta_turb=0.86,       # Turbine efficiency (optional)
    eta_nozz=0.95,       # Nozzle efficiency (optional)
    fuel_k=1.0,          # Fuel parameter (optional)
    max_iter=200,        # Maximum iterations (optional)
    tol=1e-10,           # Convergence tolerance (optional)
    damping=0.5,         # Newton damping (optional)
    mass_tol=1e-9,       # Mass tolerance (optional)
    energy_tol=1e-9,     # Energy tolerance (optional)
    t4_max=1400.0,       # Maximum T4 [K] (optional)
)

print(f"Status: {result.status}")
print(f"Converged: {result.converged}")
print(f"T4: {result.t4:.1f} K")
print(f"TSFC: {result.tsfc_proxy:.4f}")
print(f"Thrust: {result.thrust_proxy:.4f}")
```

#### `SolverResult` Class

| Attribute           | Type      | Description                   |
| ------------------- | --------- | ----------------------------- |
| `status`          | `int`   | Status code (0=OK)            |
| `converged`       | `bool`  | True if status == 0           |
| `iterations`      | `int`   | Iterations to convergence     |
| `t4`              | `float` | Turbine inlet temperature [K] |
| `tsfc_proxy`      | `float` | TSFC proxy                    |
| `thrust_proxy`    | `float` | Thrust proxy                  |
| `mass_residual`   | `float` | Final mass residual           |
| `energy_residual` | `float` | Final energy residual         |

#### `sweep(bpr_values, opr_values, ...)` → `dict`

```python
import numpy as np

bpr = np.linspace(0.2, 1.2, 21)
opr = np.linspace(4.0, 14.0, 21)

results = minotaur.sweep(bpr, opr, mach=0.65, alt_km=8.0)

# Returns dict with NumPy arrays:
# - bpr, opr: Input parameter grids
# - status, iterations: Solver results
# - t4, tsfc, thrust: Performance metrics
# - n_bpr, n_opr: Grid dimensions
```

#### `sensitivity(...)` → `dict`

```python
sens = minotaur.sensitivity(mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0)

# Returns:
# - parameters: List of parameter names
# - outputs: List of output names
# - jacobian: 6×3 NumPy array
# - base_tsfc, base_thrust, base_t4: Nominal values
```

#### `compare_degradation(...)` → `tuple`

```python
nom, deg, dtsfc, dthrust, dt4 = minotaur.compare_degradation(
    mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0,
    degradation_level="moderate"  # "light", "moderate", "severe"
)

print(f"TSFC change: {dtsfc:+.2f}%")
print(f"Thrust change: {dthrust:+.2f}%")
print(f"T4 change: {dt4:+.1f} K")
```

#### Constants

```python
minotaur.STATUS_OK              # 0
minotaur.STATUS_MAXITER         # 1
minotaur.STATUS_DIVERGED        # 2
minotaur.STATUS_INVARIANT_VIOL  # 3
minotaur.STATUS_CONSTRAINT_VIOL # 4
minotaur.STATUS_NONPHYSICAL     # 5
```

---

## Fortran API

### C Bindings

All Fortran functions are exposed via C ABI for FFI compatibility.

#### `minotaur_solve_c`

```fortran
subroutine minotaur_solve_c(inp, out) bind(C, name="minotaur_solve_c")
    type(MinotaurInput), intent(in) :: inp
    type(MinotaurOutput), intent(out) :: out
end subroutine
```

#### `minotaur_solve_ext_c`

```fortran
subroutine minotaur_solve_ext_c(inp_ext, out) bind(C, name="minotaur_solve_ext_c")
    type(MinotaurInputExt), intent(in) :: inp_ext
    type(MinotaurOutput), intent(out) :: out
end subroutine
```

#### `minotaur_solve_ad_c`

```fortran
subroutine minotaur_solve_ad_c(mach, alt_km, bpr, opr, eta_comp, eta_turb, &
                                eta_nozz, fuel_k, t4_max, seed_param, &
                                tsfc_val, tsfc_der, thrust_val, thrust_der, &
                                t4_val, t4_der, status) bind(C, name="minotaur_solve_ad_c")
```

#### `minotaur_jacobian_c`

```fortran
subroutine minotaur_jacobian_c(mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max, &
                                jacobian, base_tsfc, base_thrust, base_t4, status) &
                                bind(C, name="minotaur_jacobian_c")
```

---

## Configuration Reference

### Complete TOML Schema

```toml
[CSTNSystems]
program = "CSTNSystems"           # Program identifier (required)
module  = "minotaur"        # Module name (required)
version = "1.0.0"           # Config version (required)

[solver]
max_iter = 64               # Maximum iterations [1, 10000]
tol      = 1.0e-10          # Convergence tolerance [1e-15, 1e-3]
damping  = 0.5              # Newton damping factor (0, 1]

[invariants]
mass_tol   = 1.0e-9         # Mass conservation gate [1e-15, 1e-3]
energy_tol = 1.0e-9         # Energy conservation gate [1e-15, 1e-3]

[constraints]
t4_max = 1400.0             # Thermal ceiling [K] [800, 2000]

[cycle]
mach     = 0.65             # Flight Mach [0, 0.95]
alt_km   = 8.0              # Altitude [km] [0, 20]
bpr      = 0.6              # Bypass ratio (optional for sweep)
opr      = 8.0              # Overall pressure ratio (optional for sweep)
eta_comp = 0.82             # Compressor efficiency [0.7, 0.95]
eta_turb = 0.86             # Turbine efficiency [0.8, 0.95]
eta_nozz = 0.95             # Nozzle efficiency [0.9, 1.0]
fuel_k   = 1.0              # Fuel parameter [0.5, 2.0]

[sweep]                     # Optional: parameter sweep
bpr_min = 0.2
bpr_max = 1.2
bpr_n   = 21                # Grid points [2, 1000]
opr_min = 4.0
opr_max = 14.0
opr_n   = 21

[components]                # Optional: component models (v2.4+)
compressor = "standard"     # "standard" or "advanced"
turbine    = "standard"
nozzle     = "standard"

[losses]                    # Optional: loss coefficients (v2.4+)
inlet   = 0.02              # [0, 0.5]
burner  = 0.04
turbine = 0.02
nozzle  = 0.01

[degradation]               # Optional: degradation factors (v2.4+)
eta_comp_factor = 0.90      # [0.5, 1.0]
eta_turb_factor = 0.94
loss_adder      = 0.02      # [0, 0.1]

[physics]                   # Optional: extended physics (v2.1+)
variable_cp  = false        # Enable variable specific heats
real_gas     = false        # Enable real gas corrections
combustion   = "simple"     # "simple", "equilibrium", "kinetic"
```

---

## Status Codes

| Code | Name                | Description                | Recommended Action                  |
| ---- | ------------------- | -------------------------- | ----------------------------------- |
| 0    | `OK`              | Converged successfully     | Use results                         |
| 1    | `MAXITER`         | Iteration limit reached    | Increase max_iter or adjust damping |
| 2    | `DIVERGED`        | Residual increased 10×    | Check input feasibility             |
| 3    | `INVARIANT_VIOL`  | Conservation law violation | Tighten tolerances                  |
| 4    | `CONSTRAINT_VIOL` | T4 exceeded limit          | Reduce OPR or fuel_k                |
| 5    | `NONPHYSICAL`     | NaN/Inf detected           | Check parameter bounds              |

---

## Output Formats

### CSV Format

```csv
case,bpr,opr,mach,alt_km,status,converged,iter,mass_resid,energy_resid,t4,tsfc_proxy,thrust_proxy
baseline,0.600000,8.000000,0.6500,8.0000,0,true,12,2.1e-12,1.8e-12,1285.30,0.912340,1.034100
```

### JSON Manifest Schema

```json
{
  "schema_version": "1.0.0",
  "solver_version": "1.0.0",
  "CSTNSystems_program_id": "CSTNSystems-MINOTAUR",
  "timestamp_utc": "2023-01-19T12:00:00Z",
  "platform": "linux",
  "config_hash": "a1b2c3d4...",
  "config_snapshot": { ... }
}
```

---

*CSTNSystems - Compact Subsonic Turbofan Numerical Systems*
