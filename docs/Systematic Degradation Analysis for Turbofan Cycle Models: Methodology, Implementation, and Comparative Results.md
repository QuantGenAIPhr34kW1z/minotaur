# Systematic Degradation Analysis for Turbofan Cycle Models: Methodology, Implementation, and Comparative Results

**CSTNSystems Technical Report 2023-004**

---

## Abstract

We present a systematic methodology for analyzing performance degradation in reduced-order turbofan cycle models, implemented within the MINOTAUR framework. The methodology defines three standardized degradation scenarios-light (5% efficiency loss), moderate (10% efficiency loss), and severe (15% efficiency loss)-with corresponding pressure loss increments. Comparative analysis between nominal and degraded configurations reveals quantifiable performance impacts: moderate degradation increases TSFC by +11.1%, decreases thrust by −6.0%, and raises turbine inlet temperature by +57 K. The framework implements a `compare` subcommand enabling automated nominal-vs-degraded analysis with structured JSON output for post-processing. Validation across the 441-point parameter sweep demonstrates that degradation shifts the feasibility boundary, reducing the thermally-admissible operating envelope by 8.2% at severe degradation levels. The methodology supports prognostic studies, maintenance scheduling analysis, and robustness assessment under realistic efficiency deterioration scenarios.

**Keywords:** degradation analysis, turbofan cycle, efficiency deterioration, prognostics, reduced-order modeling, maintenance scheduling

---

## 1. Introduction

### 1.1 Motivation

Gas turbine performance degrades throughout service life due to multiple mechanisms:

1. **Compressor fouling** - Blade surface roughness increases, reducing efficiency
2. **Hot section erosion** - Turbine blade deterioration from high-temperature operation
3. **Seal wear** - Tip clearance increases, causing additional leakage losses
4. **Foreign object damage** - Discrete events causing permanent efficiency loss

Understanding degradation impacts is critical for:

- **Prognostics** - Predicting remaining useful life
- **Maintenance scheduling** - Optimizing intervention timing
- **Performance trending** - Identifying anomalous deterioration
- **Design margins** - Specifying adequate thermal headroom

### 1.2 Problem Statement

Traditional degradation analysis approaches suffer from:

1. **Ad-hoc parameterization** - Degradation levels defined inconsistently
2. **Manual comparison** - Nominal and degraded runs processed separately
3. **Implicit assumptions** - Degradation mechanisms not clearly documented
4. **Limited reproducibility** - Results difficult to replicate

### 1.3 Contributions

This work presents:

1. **Standardized degradation taxonomy** - Three documented levels with physical rationale
2. **Automated comparison workflow** - Single command generates nominal/degraded analysis
3. **Structured output format** - JSON schema with delta metrics
4. **Feasibility boundary analysis** - Quantified envelope shrinkage under degradation
5. **Sensitivity to degradation** - Jacobian of performance w.r.t. efficiency loss

---

## 2. Degradation Model

### 2.1 Degradation Mechanisms

The reduced-order model captures degradation through three parameters:

| Parameter                    | Symbol | Physical Mechanism                      |
| ---------------------------- | ------ | --------------------------------------- |
| Compressor efficiency factor | κ_c   | Fouling, erosion, clearance             |
| Turbine efficiency factor    | κ_t   | Erosion, oxidation, cooling degradation |
| Additional pressure loss     | Δζ   | Seal wear, leakage paths                |

Effective component efficiencies under degradation:

```
η_c,eff = η_c,nominal · κ_c                                        (1)
η_t,eff = η_t,nominal · κ_t                                        (2)
ζ_total = ζ_nominal + Δζ                                           (3)
```

### 2.2 Standardized Degradation Levels

Three standardized scenarios capture the degradation spectrum:

| Level    | κ_c | κ_t | Δζ | Physical Interpretation          |
| -------- | ---- | ---- | ---- | -------------------------------- |
| Light    | 0.95 | 0.97 | 0.01 | Early in-service, minor fouling  |
| Moderate | 0.90 | 0.94 | 0.02 | Mid-life, significant wear       |
| Severe   | 0.85 | 0.91 | 0.03 | End-of-life, major deterioration |

**Rationale for asymmetric factors:**

- Compressor degrades faster than turbine (κ_c < κ_t) due to environmental exposure
- Turbine operates in cleaner (post-combustion) flow but at higher temperatures
- Literature suggests κ_c/κ_t ≈ 0.95–0.98 ratio typical [1, 2]

### 2.3 Custom Degradation Configuration

Users may specify custom degradation via TOML:

```toml
[degradation]
eta_comp_factor = 0.92    # Custom compressor factor
eta_turb_factor = 0.95    # Custom turbine factor
loss_adder      = 0.015   # Custom additional losses
scenario_name   = "custom_mid"
```

Validation bounds:

- κ_c ∈ [0.5, 1.0] - Physical efficiency limits
- κ_t ∈ [0.5, 1.0] - Physical efficiency limits
- Δζ ∈ [0.0, 0.2] - Maximum reasonable additional loss

---

## 3. Methodology

### 3.1 Comparison Workflow

The `minotaur compare` command automates nominal-vs-degraded analysis:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Compare Workflow                              │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │    Config    │───▶│   Nominal    │───▶│   Nominal    │      │
│  │    (TOML)    │    │    Input     │    │   Output     │      │
│  └──────────────┘    └──────────────┘    └──────┬───────┘      │
│         │                                        │              │
│         │                                        ▼              │
│         │            ┌──────────────┐    ┌──────────────┐      │
│         │            │  Degradation │    │    Delta     │      │
│         └───────────▶│   Applied    │───▶│   Metrics    │      │
│                      └──────────────┘    └──────────────┘      │
│                             │                    │              │
│                             ▼                    ▼              │
│                      ┌──────────────┐    ┌──────────────┐      │
│                      │   Degraded   │    │   Output     │      │
│                      │    Output    │    │ (CSV/JSON)   │      │
│                      └──────────────┘    └──────────────┘      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Delta Metrics

The comparison produces four primary delta metrics:

| Metric           | Symbol  | Definition                               |
| ---------------- | ------- | ---------------------------------------- |
| TSFC change      | ΔTSFC% | (TSFC_deg − TSFC_nom) / TSFC_nom × 100 |
| Thrust change    | ΔF%    | (F_deg − F_nom) / F_nom × 100          |
| T₄ change       | ΔT₄   | T₄,deg − T₄,nom [K]                   |
| Iteration change | Δiter  | iter_deg − iter_nom                     |

### 3.3 Command Interface

```bash
# Light degradation analysis
minotaur compare --config baseline.toml --level light --json

# Moderate degradation analysis
minotaur compare --config baseline.toml --level moderate --json

# Severe degradation analysis
minotaur compare --config baseline.toml --level severe --json

# Custom degradation (from config)
minotaur compare --config custom_degraded.toml --level custom --json
```

---

## 4. Results

### 4.1 Baseline Comparison

Operating point: β = 0.6, π = 8, M = 0.65, h = 8 km

| Metric       | Nominal | Light  | Moderate | Severe |
| ------------ | ------- | ------ | -------- | ------ |
| Status       | OK      | OK     | OK       | OK     |
| Iterations   | 12      | 13     | 15       | 18     |
| T₄ [K]      | 1285.3  | 1313.1 | 1342.7   | 1374.5 |
| TSFC proxy   | 0.9123  | 0.9661 | 1.0136   | 1.0702 |
| Thrust proxy | 1.0341  | 1.0019 | 0.9721   | 0.9390 |

### 4.2 Performance Impact by Degradation Level

```
    TSFC Change (%)
         │
    +20 ─┤                              ●  Severe (+17.3%)
        │
    +15 ─┤
        │
    +10 ─┤                    ●  Moderate (+11.1%)
        │
     +5 ─┤          ●  Light (+5.9%)
        │
      0 ─┤  ●  Nominal
        │
        └────────────────────────────────────────
           Nominal   Light   Moderate   Severe
                    Degradation Level

Figure 1: TSFC degradation follows approximately linear progression
with efficiency loss. Each 5% compressor efficiency reduction adds
approximately 5.7% to TSFC.
```

```
    Thrust Change (%)
         │
      0 ─┤  ●  Nominal
        │
     −3 ─┤          ●  Light (−3.1%)
        │
     −6 ─┤                    ●  Moderate (−6.0%)
        │
     −9 ─┤                              ●  Severe (−9.2%)
        │
        └────────────────────────────────────────
           Nominal   Light   Moderate   Severe
                    Degradation Level

Figure 2: Thrust degradation is proportionally smaller than TSFC
impact but still significant at severe degradation levels.
```

```
    T₄ Change (K)
         │
    +90 ─┤                              ●  Severe (+89 K)
        │
    +60 ─┤                    ●  Moderate (+57 K)
        │
    +30 ─┤          ●  Light (+28 K)
        │
      0 ─┤  ●  Nominal
        │
        └────────────────────────────────────────
           Nominal   Light   Moderate   Severe
                    Degradation Level

Figure 3: Thermal loading increases significantly with degradation.
At severe levels, T₄ approaches constraint boundary.
```

### 4.3 Summary Table

| Degradation Level | ΔTSFC% | ΔF%  | ΔT₄ [K] | Δiter |
| ----------------- | ------- | ----- | --------- | ------ |
| Light (5%)        | +5.9    | −3.1 | +28       | +1     |
| Moderate (10%)    | +11.1   | −6.0 | +57       | +3     |
| Severe (15%)      | +17.3   | −9.2 | +89       | +6     |

**Degradation Sensitivity Coefficients:**

Linearized relationships (per 1% compressor efficiency loss):

| Metric     | Sensitivity | Units    |
| ---------- | ----------- | -------- |
| TSFC       | +1.15%      | %/%η    |
| Thrust     | −0.61%     | %/%η    |
| T₄        | +5.9 K      | K/%η    |
| Iterations | +0.4        | iter/%η |

### 4.4 Feasibility Envelope Analysis

Degradation shifts the thermal constraint boundary, shrinking the feasible operating envelope.

**Envelope Shrinkage Analysis (π at constraint boundary for β = 0.6):**

```
    Feasible OPR
         │
     11 ─┤  ●  Nominal (π* = 10.5)
        │
     10 ─┤        ●  Light (π* = 9.8)
        │
      9 ─┤              ●  Moderate (π* = 9.2)
        │
      8 ─┤                    ●  Severe (π* = 8.5)
        │
        └────────────────────────────────────────
           Nominal   Light   Moderate   Severe
                    Degradation Level

Figure 4: Maximum feasible OPR decreases with degradation due to
increased thermal loading at each operating point.
```

**Feasible Region Comparison (441-point sweep):**

| Degradation | Converged | Constrained | Envelope Size  |
| ----------- | --------- | ----------- | -------------- |
| Nominal     | 418       | 23          | 94.7%          |
| Light       | 411       | 30          | 93.2% (−1.5%) |
| Moderate    | 402       | 39          | 91.2% (−3.5%) |
| Severe      | 382       | 59          | 86.5% (−8.2%) |

```
                         OPR (π)
                              │
Nominal:    14 ─┤ █ █ █ █ █ █ █ █ █ █ █ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░
                │
Moderate:   14 ─┤ █ █ █ █ █ █ █ █ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░
                │
Severe:     14 ─┤ █ █ █ █ █ █ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░
                │
                └─────────────────────────────────────────────
                  0.2   0.4   0.6   0.8   1.0   1.2
                                BPR (β)

Figure 5: Feasibility envelope comparison at OPR = 14 row.
Degradation progressively shrinks the thermally-admissible region.
Legend: █ = Converged, ░ = T₄ violated
```

### 4.5 Iteration Count Distribution Shift

```
    Frequency
         │
     ▓▓▓▓│                          Nominal (mean=14.3)
     ░░░░│░░░░
         │    ░░░░ ▓▓▓▓
         │         ▓▓▓▓ ░░░░      Light (mean=15.1)
         │              ░░░░ ▓▓▓▓
         │                   ▓▓▓▓ ░░░░
         │                        ░░░░    Moderate (mean=16.8)
         │                             ▓▓▓▓
         │                                  ░░░░    Severe (mean=19.2)
         └─────────────────────────────────────────────
            8    12    16    20    24    28    32
                       Iteration Count

Figure 6: Iteration distribution shifts right with increasing
degradation. The solver requires more iterations to converge
as the problem becomes more nonlinear.
```

| Degradation | Mean Iter | Std Dev | Max |
| ----------- | --------- | ------- | --- |
| Nominal     | 14.3      | 6.2     | 47  |
| Light       | 15.1      | 6.8     | 51  |
| Moderate    | 16.8      | 7.4     | 58  |
| Severe      | 19.2      | 8.9     | 72  |

---

## 5. Applications

### 5.1 Prognostic Analysis

The degradation model supports remaining useful life estimation:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Prognostic Workflow                          │
│                                                                  │
│  1. Measure current performance                                  │
│     └─▶ Extract effective η_c, η_t from operational data        │
│                                                                  │
│  2. Compute current degradation level                            │
│     └─▶ κ_c = η_c,measured / η_c,nominal                        │
│                                                                  │
│  3. Project future thermal margin                                │
│     └─▶ Run comparison at projected degradation                 │
│                                                                  │
│  4. Identify intervention threshold                              │
│     └─▶ T₄,projected → T₄,max triggers maintenance alert        │
└─────────────────────────────────────────────────────────────────┘
```

### 5.2 Maintenance Scheduling

Economic optimization of maintenance timing:

| Degradation Level | Fuel Penalty | Thrust Margin | Thermal Margin | Recommendation    |
| ----------------- | ------------ | ------------- | -------------- | ----------------- |
| Light (5%)        | +5.9%        | 96.9%         | 87 K           | Monitor           |
| Moderate (10%)    | +11.1%       | 94.0%         | 57 K           | Plan intervention |
| Severe (15%)      | +17.3%       | 90.8%         | 26 K           | Immediate action  |

### 5.3 Design Margin Assessment

The degradation analysis quantifies required design margins:

```
    Required T₄ Margin (K)
         │
    100 ─┤          ●  For severe tolerance
        │
     75 ─┤    ●  For moderate tolerance
        │
     50 ─┤
        │  ●  For light tolerance
     25 ─┤
        │
        └────────────────────────────────────────
             Light    Moderate    Severe
                  Design Tolerance

Figure 7: Thermal design margin required to maintain feasibility
throughout the degradation life cycle.
```

---

## 6. Implementation

### 6.1 Degradation Application Logic

```fortran
! Apply degradation factors to effective efficiencies
eta_comp_eff = inp_ext%eta_comp * inp_ext%eta_comp_factor
eta_turb_eff = inp_ext%eta_turb * inp_ext%eta_turb_factor

! Clamp to valid range
eta_comp_eff = max(0.5, min(1.0, eta_comp_eff))
eta_turb_eff = max(0.5, min(1.0, eta_turb_eff))

! Apply loss adder
loss_factor = 1.0 - (inlet_loss + burner_loss + turb_loss +
                     nozzle_loss + loss_adder)
```

### 6.2 JSON Output Schema

```json
{
  "manifest": {
    "schema_version": "1.0.0",
    "solver_version": "0.5.0",
    "CSTNSystems_program_id": "CSTNSystems-MINOTAUR"
  },
  "nominal": {
    "scenario_name": "nominal",
    "status": 0,
    "converged": true,
    "iterations": 12,
    "t4": 1285.3,
    "tsfc_proxy": 0.9123,
    "thrust_proxy": 1.0341,
    "eta_comp_effective": 0.82,
    "eta_turb_effective": 0.86
  },
  "degraded": {
    "scenario_name": "degraded_moderate",
    "status": 0,
    "converged": true,
    "iterations": 15,
    "t4": 1342.7,
    "tsfc_proxy": 1.0136,
    "thrust_proxy": 0.9721,
    "eta_comp_effective": 0.738,
    "eta_turb_effective": 0.8084
  },
  "delta": {
    "tsfc_change_pct": 11.11,
    "thrust_change_pct": -6.00,
    "t4_change_k": 57.4,
    "iter_change": 3
  }
}
```

### 6.3 CSV Output Format

```csv
scenario,status,converged,iter,t4,tsfc_proxy,thrust_proxy,eta_comp_eff,eta_turb_eff
nominal,0,true,12,1285.30,0.912300,1.034100,0.8200,0.8600
degraded_moderate,0,true,15,1342.70,1.013600,0.972100,0.7380,0.8084

# Delta metrics
# TSFC change: 11.11%
# Thrust change: -6.00%
# T4 change: 57.4 K
# Iteration change: 3
```

---

## 7. Reproducibility

### 7.1 Execution Commands

```bash
# Build
cd src/fortran && fpm build --profile release
cd ../rust && cargo build --release

# Run degradation comparison
./target/release/minotaur compare \
    --config configs/baseline.toml \
    --level moderate \
    --out results/comparison_moderate.csv \
    --json

# Run all degradation levels
for level in light moderate severe; do
    ./target/release/minotaur compare \
        --config configs/baseline.toml \
        --level $level \
        --out results/comparison_${level}.csv \
        --json
done
```

### 7.2 Verification

```bash
# Verify degraded results differ from nominal
minotaur run --config configs/baseline.toml --out nominal.csv
minotaur compare --config configs/baseline.toml --level moderate

# Check T4 increases (degraded > nominal)
awk -F, 'NR==2 {nom=$5} NR==3 {deg=$5; if(deg>nom) print "OK: T4 increased"} ' \
    results/comparison_moderate.csv
```

### 7.3 Determinism Verification

```bash
# Multiple runs should produce identical results
minotaur compare --config baseline.toml --level moderate --out run1.csv
minotaur compare --config baseline.toml --level moderate --out run2.csv
diff run1.csv run2.csv  # Must be empty
```

---

## 8. Conclusions

The systematic degradation analysis methodology provides:

1. **Standardized scenarios** - Three documented levels with physical rationale
2. **Quantified impacts** - TSFC +11.1%, Thrust −6.0%, T₄ +57 K at moderate degradation
3. **Envelope shrinkage** - 8.2% feasible region reduction at severe degradation
4. **Automated workflow** - Single command comparison with structured output
5. **Application support** - Prognostics, maintenance scheduling, design margins

The linear degradation sensitivities (1.15%/%η for TSFC, 5.9 K/%η for T₄) enable first-order impact estimation for arbitrary efficiency deterioration levels.

---

## Acknowledgments

This work was conducted under the CSTNSystems research program. The degradation model parameters are representative values based on published literature; actual values vary by engine type and operating environment.

---

## References

[1] Diakunchak, I.S., "Performance Deterioration in Industrial Gas Turbines," J. Eng. Gas Turbines Power, vol. 114, no. 2, pp. 161-168, 1992.

[2] Kurz, R., Brun, K., "Degradation in Gas Turbine Systems," J. Eng. Gas Turbines Power, vol. 123, no. 1, pp. 70-77, 2001.

[3] Li, Y.G., "Performance-analysis-based gas turbine diagnostics: A review," Proc. Inst. Mech. Eng. A J. Power Energy, vol. 216, no. 5, pp. 363-377, 2002.

[4] Volponi, A.J., "Gas Turbine Engine Health Management: Past, Present, and Future Trends," J. Eng. Gas Turbines Power, vol. 136, no. 5, 2014.

[5] Saravanamuttoo, H.I.H., et al., "Gas Turbine Theory," 7th ed., Pearson, 2017.

[6] Walsh, P.P., Fletcher, P., "Gas Turbine Performance," 2nd ed., Blackwell, 2004.

---

*CSTNSystems - Compact Subsonic Turbofan Numerical Systems*
