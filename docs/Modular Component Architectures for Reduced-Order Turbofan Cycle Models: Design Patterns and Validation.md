# Modular Component Architectures for Reduced-Order Turbofan Cycle Models: Design Patterns and Validation

**CSTNSystems Technical Report 2023-003**

---

## Abstract

We present a modular component architecture for reduced-order turbofan cycle models that enables systematic comparison of sub-model formulations without requiring solver modifications. The architecture introduces abstraction layers for compressor, turbine, and nozzle components, each supporting two model variants: a standard isentropic efficiency formulation and an advanced model incorporating polytropic effects, cooling flows, and divergence losses. The design employs explicit loss coefficient parameterization for inlet (ζ_inlet = 0.02), combustor (ζ_burner = 0.04), turbine mechanical (ζ_turb = 0.02), and nozzle velocity (ζ_nozzle = 0.01) losses, all configurable via the TOML interface. Validation across 441 parameter sweep points demonstrates consistent convergence behavior (94.7%) for both model variants, with the advanced formulation producing systematically higher thermal loads (+2.1% T₄) and slightly reduced thrust (−1.5%) due to cooling and divergence penalties. The architecture maintains deterministic execution and backward compatibility with existing configurations while enabling comparative model fidelity studies.

**Keywords:** modular architecture, component models, turbofan cycle, loss coefficients, polytropic efficiency, reduced-order modeling

---

## 1. Introduction

### 1.1 Motivation

Reduced-order turbofan cycle models necessarily simplify component-level physics. The choice of sub-model formulation-isentropic vs. polytropic efficiency, lumped vs. distributed losses, ideal vs. cooled turbines-directly impacts predicted performance metrics. Traditional monolithic implementations make systematic comparison difficult:

1. **Equation coupling** - Component interactions are embedded in solver logic
2. **Hard-coded constants** - Loss coefficients scattered through source code
3. **Model switching** - Requires code modification and recompilation
4. **Validation overhead** - Each change requires full regression testing

### 1.2 Contributions

This work presents:

1. **Component abstraction layer** - Compressor, turbine, nozzle interfaces
2. **Two-tier model hierarchy** - Standard (isentropic) and advanced (polytropic/cooled)
3. **Explicit loss parameterization** - All loss coefficients exposed via configuration
4. **Configuration-driven selection** - Model variants specified in TOML without recompilation
5. **Validation methodology** - Comparative sweep analysis across model variants

### 1.3 Scope

The architecture supports reduced-order thermodynamic modeling only. It does not include:

- Geometry-dependent correlations
- Off-design component maps
- Transient behavior
- Multi-spool dynamics

---

## 2. Component Architecture

### 2.1 Design Principles

The modular architecture follows established software engineering patterns:

| Principle               | Implementation                                    |
| ----------------------- | ------------------------------------------------- |
| Single Responsibility   | Each component handles one transformation         |
| Open/Closed             | New models add, don't modify existing code        |
| Dependency Inversion    | Solver depends on interfaces, not implementations |
| Configuration over Code | Model selection via TOML, not recompilation       |

### 2.2 Component Interfaces

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Cycle Solver                                  │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                │
│   │ Compressor  │  │   Turbine   │  │   Nozzle    │                │
│   │  Interface  │──│  Interface  │──│  Interface  │                │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                │
│          │                │                │                        │
│    ┌─────┴─────┐    ┌─────┴─────┐    ┌─────┴─────┐                 │
│    ▼           ▼    ▼           ▼    ▼           ▼                 │
│ ┌──────┐  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐            │
│ │ Std. │  │ Adv. │ │ Std. │ │ Adv. │ │ Std. │ │ Adv. │            │
│ │Model │  │Model │ │Model │ │Model │ │Model │ │Model │            │
│ └──────┘  └──────┘ └──────┘ └──────┘ └──────┘ └──────┘            │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.3 Configuration Schema

```toml
[components]
compressor = "standard"   # or "advanced"
turbine    = "standard"   # or "advanced"
nozzle     = "standard"   # or "advanced"

[losses]
inlet   = 0.02            # Inlet pressure loss coefficient
burner  = 0.04            # Combustor pressure loss
turbine = 0.02            # Turbine mechanical loss
nozzle  = 0.01            # Nozzle velocity loss coefficient
```

---

## 3. Component Models

### 3.1 Compressor Models

**Standard Model (Isentropic Efficiency)**

The standard compressor model uses a single-stage isentropic efficiency formulation:

```
T₂/T₁ = 1 + (π_c^((γ-1)/γ) - 1) / η_c                              (1)
```

where:

- T₂, T₁ = outlet, inlet temperatures [K]
- π_c = compressor pressure ratio [−]
- γ = specific heat ratio (1.4 for air)
- η_c = isentropic efficiency [−]

**Advanced Model (Polytropic Efficiency)**

The advanced model employs multi-stage polytropic analysis:

```
T₂/T₁ = π_c^((γ-1)/(γ·η_p))                                        (2)
```

with stage-by-stage integration for N stages:

```
T_{i+1} = T_i · (π_stage)^((γ-1)/(γ·η_p))                          (3)
π_stage = π_c^(1/N)                                                 (4)
```

The polytropic formulation better captures:

- Stage stacking effects
- Reheating between stages
- More realistic temperature rise at high pressure ratios

### 3.2 Turbine Models

**Standard Model (Isentropic Efficiency)**

```
T₄/T₅ = 1 - η_t · (1 - π_t^(-(γ_g-1)/γ_g))                         (5)
```

where:

- γ_g = 1.33 (hot gas specific heat ratio)
- π_t = turbine expansion ratio [−]
- η_t = isentropic efficiency [−]

**Advanced Model (Cooled Turbine)**

The advanced model includes cooling flow effects:

```
T₄/T₅ = 1 - η_t · f_cool · (1 - π_t^(-(γ_g-1)/γ_g))                (6)

f_cool = 0.98  if T₄ > T_metal,max                                  (7)
       = 1.00  otherwise
```

The cooling factor accounts for:

- Coolant bleed extraction (performance penalty)
- Metal temperature limits
- Film cooling effectiveness

### 3.3 Nozzle Models

**Standard Model (Velocity Coefficient)**

```
V_exit = η_n · √(2·c_p·T_in·(1 - (1/π_n)^((γ-1)/γ)))               (8)
```

where:

- η_n = velocity coefficient [−]
- c_p = specific heat capacity [J/(kg·K)]
- π_n = nozzle pressure ratio [−]

**Advanced Model (Divergence Losses)**

The advanced model includes conical divergence losses:

```
V_exit = η_n · f_div · √(2·c_p·T_in·(1 - (1/π_n)^((γ-1)/γ)))       (9)

f_div = 0.5 · (1 + cos(θ_half))                                    (10)
```

where θ_half ≈ 15° is the half-angle of the conical nozzle.

---

## 4. Loss Coefficient Framework

### 4.1 Loss Taxonomy

| Loss Type | Symbol    | Default | Valid Range | Physical Mechanism              |
| --------- | --------- | ------- | ----------- | ------------------------------- |
| Inlet     | ζ_inlet  | 0.02    | [0, 0.5]    | Diffuser and duct friction      |
| Burner    | ζ_burner | 0.04    | [0, 0.5]    | Combustor pressure drop         |
| Turbine   | ζ_turb   | 0.02    | [0, 0.5]    | Mechanical and tip losses       |
| Nozzle    | ζ_nozzle | 0.01    | [0, 0.5]    | Velocity profile non-uniformity |

### 4.2 Loss Application

Total effective efficiency includes accumulated losses:

```
η_eff = η_base · (1 - Σζ_i)                                        (11)
```

where the loss factor multiplies the nozzle efficiency to capture system-level pressure losses.

### 4.3 Loss Sensitivity

Partial derivatives of performance metrics with respect to loss coefficients (at nominal conditions):

| Metric | ∂/∂ζ_inlet | ∂/∂ζ_burner | ∂/∂ζ_turb | ∂/∂ζ_nozzle |
| ------ | ------------- | -------------- | ------------ | -------------- |
| TSFC   | +0.12         | +0.18          | +0.09        | +0.05          |
| Thrust | −0.08        | −0.12         | −0.06       | −0.04         |
| T₄    | +2.1          | +3.4           | +1.2         | +0.8           |

**Interpretation:** Burner losses have the largest performance impact, followed by inlet losses. This hierarchy reflects the thermodynamic significance of pressure losses at different cycle stations.

---

## 5. Validation Methodology

### 5.1 Experimental Design

Comparative validation using a 441-point parameter sweep:

| Configuration | Compressor | Turbine  | Nozzle   | Losses  |
| ------------- | ---------- | -------- | -------- | ------- |
| Baseline (A)  | standard   | standard | standard | default |
| Advanced (B)  | advanced   | advanced | advanced | default |
| Custom (C)    | advanced   | standard | advanced | custom  |

Sweep parameters: β ∈ [0.2, 1.2], π ∈ [4, 14], 21×21 grid.

### 5.2 Convergence Comparison

| Model Configuration | Converged | Constrained | Convergence Rate |
| ------------------- | --------- | ----------- | ---------------- |
| Standard (A)        | 418       | 23          | 94.7%            |
| Advanced (B)        | 416       | 25          | 94.3%            |
| Custom (C)          | 417       | 24          | 94.6%            |

All configurations exhibit similar convergence behavior, validating that the modular architecture does not introduce numerical instabilities.

### 5.3 Performance Comparison

**Thermal Proxy (T₄) at β = 0.6, π = 8:**

```
    T₄ (K)
         │
  1310 ─┤                        ● Advanced (+2.1%)
        │
  1300 ─┤
        │
  1290 ─┤            ● Standard (baseline)
        │
  1280 ─┤
        │
        └────────────────────────────────────────
              Std.      Adv.      Custom
                    Model Configuration

Figure 1: T₄ comparison across model configurations. Advanced models
produce higher thermal loads due to polytropic effects and cooling penalties.
```

**TSFC Proxy Comparison:**

```
    TSFC
         │
   0.95 ─┤  ● Standard
        │
   0.94 ─┤
        │        ● Advanced (+0.8%)
   0.93 ─┤
        │              ● Custom
   0.92 ─┤
        │
        └────────────────────────────────────────
              Std.      Adv.      Custom
                    Model Configuration

Figure 2: TSFC comparison. Advanced models show slightly higher fuel
consumption due to efficiency penalties.
```

**Thrust Proxy Comparison:**

```
    Thrust
         │
   1.05 ─┤  ● Standard
        │
   1.04 ─┤
        │
   1.03 ─┤        ● Advanced (−1.5%)
        │
   1.02 ─┤              ● Custom
        │
        └────────────────────────────────────────
              Std.      Adv.      Custom
                    Model Configuration

Figure 3: Thrust comparison. Advanced models show reduced thrust due
to cooling bleed and divergence losses.
```

### 5.4 Iteration Count Comparison

```
    Mean Iterations
         │
     16 ─┤                    ● Advanced
        │
     15 ─┤              ● Custom
        │
     14 ─┤  ● Standard
        │
     13 ─┤
        │
        └────────────────────────────────────────
              Std.      Adv.      Custom
                    Model Configuration

Figure 4: Mean iteration comparison. Advanced models require slightly
more iterations due to increased nonlinearity.
```

| Configuration | Mean Iter | Std Dev | Min | Max |
| ------------- | --------- | ------- | --- | --- |
| Standard (A)  | 14.3      | 6.2     | 8   | 47  |
| Advanced (B)  | 15.8      | 7.1     | 9   | 52  |
| Custom (C)    | 15.2      | 6.7     | 8   | 49  |

---

## 6. Model Selection Guidelines

### 6.1 Decision Matrix

| Use Case            | Recommended Model | Rationale                               |
| ------------------- | ----------------- | --------------------------------------- |
| Quick screening     | Standard          | Faster convergence, sufficient accuracy |
| Detailed design     | Advanced          | Better physics, higher fidelity         |
| Optimization        | Standard          | Lower computational cost per iteration  |
| Constraint analysis | Advanced          | More accurate thermal predictions       |
| Comparative studies | Both              | Quantify model uncertainty              |

### 6.2 Computational Considerations

| Metric                 | Standard | Advanced | Ratio     |
| ---------------------- | -------- | -------- | --------- |
| Wall time (single run) | 1.0×    | 1.12×   | +12%      |
| Iterations (mean)      | 14.3     | 15.8     | +10%      |
| Memory footprint       | 1.0×    | 1.0×    | No change |

The advanced models impose modest computational overhead suitable for design studies.

### 6.3 Fidelity Assessment

```
    Model Fidelity
         │
   High ─┤                        ● CFD (out of scope)
        │
        │                  ● Advanced models
        │
   Med. ─┤            ● Standard models
        │
        │      ● Textbook correlations
        │
   Low ─┤ ● Back-of-envelope
        │
        └────────────────────────────────────────
             Low                           High
                    Computational Cost

Figure 5: Fidelity vs. cost positioning. MINOTAUR occupies the
reduced-order niche between textbook and CFD methods.
```

---

## 7. Implementation Details

### 7.1 Fortran Module Structure

```
components.f90
├── MODEL_STANDARD (0)
├── MODEL_ADVANCED (1)
├── ComponentConfig type
│   ├── compressor_model
│   ├── turbine_model
│   ├── nozzle_model
│   └── loss coefficients
├── compressor_standard()
├── compressor_advanced()
├── turbine_standard()
├── turbine_advanced()
├── nozzle_standard()
├── nozzle_advanced()
├── apply_degradation_eta()
└── apply_degradation_loss()
```

### 7.2 FFI Extension

The extended FFI structure `MinotaurInputExt` adds 12 fields to the base input:

| Field             | Type   | Description            |
| ----------------- | ------ | ---------------------- |
| compressor_model  | int    | Model selector (0/1)   |
| turbine_model     | int    | Model selector (0/1)   |
| nozzle_model      | int    | Model selector (0/1)   |
| inlet_loss        | double | ζ_inlet coefficient   |
| burner_loss       | double | ζ_burner coefficient  |
| turbine_mech_loss | double | ζ_turb coefficient    |
| nozzle_loss       | double | ζ_nozzle coefficient  |
| eta_comp_factor   | double | Degradation multiplier |
| eta_turb_factor   | double | Degradation multiplier |
| loss_adder        | double | Additional losses      |
| is_degraded       | int    | Degradation flag       |

### 7.3 Backward Compatibility

Existing configurations without `[components]` or `[losses]` sections use defaults:

- All models default to "standard"
- All losses default to documented values
- The original `minotaur_solve_c()` function remains unchanged

---

## 8. Reproducibility

### 8.1 Configuration Examples

**Standard Model Configuration:**

```toml
[components]
compressor = "standard"
turbine    = "standard"
nozzle     = "standard"
```

**Advanced Model Configuration:**

```toml
[components]
compressor = "advanced"
turbine    = "advanced"
nozzle     = "advanced"

[losses]
inlet   = 0.02
burner  = 0.04
turbine = 0.02
nozzle  = 0.01
```

### 8.2 Execution Commands

```bash
# Standard model sweep
minotaur sweep --config configs/sweep_standard.toml \
    --out results/sweep_standard.csv --json

# Advanced model sweep
minotaur sweep --config configs/sweep_advanced.toml \
    --out results/sweep_advanced.csv --json

# Comparison analysis
diff results/sweep_standard.csv results/sweep_advanced.csv | head -20
```

### 8.3 Verification

```bash
# Verify model selection affects output
minotaur run --config configs/baseline_standard.toml --out std.csv
minotaur run --config configs/baseline_advanced.toml --out adv.csv

# T₄ should differ by approximately 2%
awk -F, 'NR==2 {print $12}' std.csv adv.csv
```

---

## 9. Conclusions

The modular component architecture for MINOTAUR provides:

1. **Clean separation** - Component models decoupled from solver logic
2. **Configuration-driven selection** - No recompilation for model switching
3. **Explicit parameterization** - All loss coefficients exposed and documented
4. **Validated consistency** - Both model variants maintain 94%+ convergence
5. **Quantified differences** - Advanced models show +2.1% T₄, −1.5% thrust

The architecture enables systematic model comparison and supports uncertainty quantification studies by running identical operating points through different physical formulations.

---

## Acknowledgments

This work was conducted under the CSTNSystems research program. The modular architecture design follows software engineering best practices adapted for scientific computing contexts.

---

## References

[1] Mattingly, J.D., Heiser, W.H., Pratt, D.T., "Aircraft Engine Design," 2nd ed., AIAA, 2002.

[2] Walsh, P.P., Fletcher, P., "Gas Turbine Performance," 2nd ed., Blackwell, 2004.

[3] Cumpsty, N., Heyes, A., "Jet Propulsion," 3rd ed., Cambridge, 2015.

[4] Saravanamuttoo, H.I.H., et al., "Gas Turbine Theory," 7th ed., Pearson, 2017.

[5] Kurzke, J., "Component matching for gas turbine performance analysis," J. Propulsion Power, vol. 17, no. 3, 2001.

---

*CSTNSystems - Compact Subsonic Turbofan Numerical Systems*
