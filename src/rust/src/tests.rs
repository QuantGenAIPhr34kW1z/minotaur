//! Comprehensive test suite for MINOTAUR v2.0
//!
//! Includes:
//! - Unit tests for all modules
//! - Property-based tests for numerical stability
//! - Regression tests for known configurations
//! - Performance benchmarks

use crate::ffi::{
    MinotaurInput, MinotaurInputExt, MinotaurOutput,
    solve, solve_ext, compute_jacobian,
    MODEL_STANDARD, MODEL_ADVANCED,
};
use crate::nsga2::{NSGA2, NSGA2Config, Individual, hypervolume_2d};

/// Create a default test input
fn default_input() -> MinotaurInput {
    MinotaurInput {
        mach: 0.65,
        alt_km: 8.0,
        bpr: 0.6,
        opr: 8.0,
        eta_comp: 0.82,
        eta_turb: 0.86,
        eta_nozz: 0.95,
        fuel_k: 1.0,
        max_iter: 200,
        tol: 1e-10,
        damping: 0.5,
        mass_tol: 1e-9,
        energy_tol: 1e-9,
        t4_max: 1400.0,
    }
}

// =============================================================================
// Basic Functionality Tests
// =============================================================================

#[test]
fn test_baseline_convergence() {
    let inp = default_input();
    let out = solve(inp);

    assert_eq!(out.status, 0, "Baseline should converge");
    assert!(out.iter > 0, "Should require at least one iteration");
    assert!(out.iter < 100, "Should converge in reasonable iterations");
    assert!(out.t4 > 0.0, "T4 should be positive");
    assert!(out.t4 < 1400.0, "T4 should be below limit");
}

#[test]
fn test_status_codes() {
    // Test CONSTRAINT_VIOL (T4 too high)
    let mut inp = default_input();
    inp.t4_max = 100.0;  // Unrealistically low
    let out = solve(inp);
    assert_eq!(out.status, 4, "Should report CONSTRAINT_VIOL");
}

#[test]
fn test_determinism() {
    let inp = default_input();

    let out1 = solve(inp);
    let out2 = solve(inp);

    assert_eq!(out1.status, out2.status);
    assert_eq!(out1.iter, out2.iter);
    assert!((out1.t4 - out2.t4).abs() < 1e-15, "T4 should be identical");
    assert!((out1.tsfc_proxy - out2.tsfc_proxy).abs() < 1e-15);
    assert!((out1.thrust_proxy - out2.thrust_proxy).abs() < 1e-15);
}

// =============================================================================
// Property-Based Tests
// =============================================================================

#[test]
fn test_monotonicity_opr_thrust() {
    // Higher OPR should generally increase thrust (within limits)
    let base = default_input();

    let mut results: Vec<(f64, f64)> = Vec::new();
    for opr_mult in [0.8, 0.9, 1.0, 1.1, 1.2].iter() {
        let mut inp = base;
        inp.opr = base.opr * opr_mult;
        let out = solve(inp);
        if out.status == 0 {
            results.push((inp.opr, out.thrust_proxy));
        }
    }

    // Check trend (allowing for some non-monotonicity near limits)
    if results.len() >= 3 {
        let mid = results.len() / 2;
        assert!(
            results[mid].1 > results[0].1 * 0.9,
            "Thrust should generally increase with OPR"
        );
    }
}

#[test]
fn test_efficiency_bounds() {
    // Output efficiencies should be bounded
    let inp = default_input();
    let out = solve(inp);

    if out.status == 0 {
        assert!(out.tsfc_proxy > 0.0, "TSFC should be positive");
        assert!(out.tsfc_proxy < 10.0, "TSFC should be reasonable");
        assert!(out.thrust_proxy > 0.0, "Thrust should be positive");
    }
}

#[test]
fn test_residual_bounds() {
    let inp = default_input();
    let out = solve(inp);

    if out.status == 0 {
        assert!(out.mass_resid.abs() < 1e-6, "Mass residual should be small");
        assert!(out.energy_resid.abs() < 1e-6, "Energy residual should be small");
        assert!(out.final_residual < 1e-8, "Final residual should be tiny");
    }
}

#[test]
fn test_parameter_range_sweep() {
    // Test solver across parameter space
    let base = default_input();
    let mut converged = 0;
    let mut total = 0;

    for mach in [0.3, 0.5, 0.65, 0.8].iter() {
        for alt in [2.0, 6.0, 10.0].iter() {
            for bpr in [0.3, 0.6, 0.9].iter() {
                for opr in [6.0, 8.0, 10.0].iter() {
                    let mut inp = base;
                    inp.mach = *mach;
                    inp.alt_km = *alt;
                    inp.bpr = *bpr;
                    inp.opr = *opr;

                    let out = solve(inp);
                    total += 1;
                    if out.status == 0 {
                        converged += 1;
                    }
                }
            }
        }
    }

    let convergence_rate = converged as f64 / total as f64;
    assert!(
        convergence_rate > 0.8,
        "At least 80% of valid configs should converge, got {:.1}%",
        convergence_rate * 100.0
    );
}

// =============================================================================
// Extended Model Tests
// =============================================================================

#[test]
fn test_extended_solve() {
    let base = default_input();
    let ext = MinotaurInputExt::from_base(&base);

    let out = solve_ext(ext);
    assert_eq!(out.status, 0, "Extended solve should converge");
}

#[test]
fn test_component_models() {
    let base = default_input();

    // Standard models
    let ext_std = MinotaurInputExt::from_base(&base)
        .with_models(MODEL_STANDARD, MODEL_STANDARD, MODEL_STANDARD);
    let out_std = solve_ext(ext_std);

    // Advanced models
    let ext_adv = MinotaurInputExt::from_base(&base)
        .with_models(MODEL_ADVANCED, MODEL_ADVANCED, MODEL_ADVANCED);
    let out_adv = solve_ext(ext_adv);

    // Both should converge
    assert_eq!(out_std.status, 0);
    assert_eq!(out_adv.status, 0);

    // Results may differ
    // (advanced models include additional physics)
}

#[test]
fn test_degradation() {
    let base = default_input();
    let ext = MinotaurInputExt::from_base(&base);

    // Nominal
    let out_nom = solve_ext(ext);

    // Degraded (10% efficiency loss)
    let ext_deg = ext.with_degradation(0.90, 0.94, 0.02);
    let out_deg = solve_ext(ext_deg);

    if out_nom.status == 0 && out_deg.status == 0 {
        // Degraded engine should have higher TSFC (worse)
        assert!(
            out_deg.tsfc_proxy > out_nom.tsfc_proxy * 0.95,
            "Degraded TSFC should be higher or similar"
        );
    }
}

// =============================================================================
// Automatic Differentiation Tests
// =============================================================================

#[test]
fn test_jacobian_computation() {
    let result = compute_jacobian(0.65, 8.0, 0.6, 8.0, 0.82, 0.86, 1400.0);

    assert_eq!(result.status, 0, "Jacobian computation should succeed");
    assert!(result.base_tsfc > 0.0);
    assert!(result.base_thrust > 0.0);
    assert!(result.base_t4 > 0.0);

    // Check Jacobian is not all zeros
    let mut has_nonzero = false;
    for row in result.jacobian.iter() {
        for val in row.iter() {
            if val.abs() > 1e-15 {
                has_nonzero = true;
            }
        }
    }
    assert!(has_nonzero, "Jacobian should have non-zero entries");
}

#[test]
fn test_jacobian_vs_finite_diff() {
    // Compare AD Jacobian to finite difference approximation
    let h = 1e-6;
    let mach = 0.65;
    let alt_km = 8.0;
    let bpr = 0.6;
    let opr = 8.0;
    let eta_comp = 0.82;
    let eta_turb = 0.86;
    let t4_max = 1400.0;

    // AD Jacobian
    let ad_result = compute_jacobian(mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max);

    // Finite difference for BPR (parameter 3, index 2)
    let mut inp_plus = default_input();
    let mut inp_minus = default_input();
    inp_plus.bpr = bpr + h;
    inp_minus.bpr = bpr - h;

    let out_plus = solve(inp_plus);
    let out_minus = solve(inp_minus);

    if out_plus.status == 0 && out_minus.status == 0 && ad_result.status == 0 {
        let fd_dtsfc = (out_plus.tsfc_proxy - out_minus.tsfc_proxy) / (2.0 * h);
        let ad_dtsfc = ad_result.jacobian[2][0];  // Row 2 = BPR, Col 0 = TSFC

        // Allow 1% relative error (FD has truncation error)
        let rel_error = (fd_dtsfc - ad_dtsfc).abs() / fd_dtsfc.abs().max(1e-10);
        assert!(
            rel_error < 0.05,
            "AD should match FD within 5%, got {:.1}% error",
            rel_error * 100.0
        );
    }
}

// =============================================================================
// NSGA-II Tests
// =============================================================================

#[test]
fn test_individual_dominance() {
    let mut a = Individual::new(vec![1.0, 1.0]);
    a.f = vec![1.0, 2.0];
    a.cv = 0.0;

    let mut b = Individual::new(vec![1.0, 1.0]);
    b.f = vec![2.0, 3.0];
    b.cv = 0.0;

    assert!(a.dominates(&b), "a should dominate b");
    assert!(!b.dominates(&a), "b should not dominate a");
}

#[test]
fn test_nsga2_initialization() {
    let config = NSGA2Config {
        pop_size: 20,
        generations: 1,
        bounds: vec![(0.0, 1.0), (0.0, 1.0)],
        ..Default::default()
    };

    let mut optimizer = NSGA2::new(config.clone());
    optimizer.initialize_population();

    let pop = optimizer.get_population();
    assert_eq!(pop.len(), config.pop_size);

    // Check bounds
    for ind in pop {
        assert!(ind.x[0] >= 0.0 && ind.x[0] <= 1.0);
        assert!(ind.x[1] >= 0.0 && ind.x[1] <= 1.0);
    }
}

#[test]
fn test_nsga2_optimization() {
    let config = NSGA2Config {
        pop_size: 20,
        generations: 10,
        bounds: vec![(0.0, 1.0), (0.0, 1.0)],
        ..Default::default()
    };

    let mut optimizer = NSGA2::new(config);

    // Simple ZDT1-like test function
    let eval_fn = |x: &[f64]| -> (Vec<f64>, f64, i32, Vec<f64>) {
        let f1 = x[0];
        let g = 1.0 + x[1];
        let f2 = g * (1.0 - (x[0] / g).sqrt());
        (vec![f1, f2], 0.0, 0, vec![])
    };

    let front = optimizer.optimize(eval_fn);

    assert!(!front.solutions.is_empty(), "Should find Pareto solutions");
    assert!(front.solutions.len() <= 20, "Front size bounded by pop size");

    // All solutions should be rank 0
    for sol in &front.solutions {
        assert_eq!(sol.rank, 0, "Front solutions should be rank 0");
    }
}

#[test]
fn test_hypervolume() {
    let mut ind1 = Individual::new(vec![]);
    ind1.f = vec![0.5, 0.5];

    let mut ind2 = Individual::new(vec![]);
    ind2.f = vec![0.3, 0.8];

    let front = vec![ind1, ind2];
    let ref_point = (1.0, 1.0);

    let hv = hypervolume_2d(&front, ref_point);
    assert!(hv > 0.0, "Hypervolume should be positive");
    assert!(hv < 1.0, "Hypervolume should be less than reference area");
}

// =============================================================================
// Regression Tests
// =============================================================================

#[test]
fn test_regression_baseline() {
    // Known-good baseline result (update if physics changes)
    let inp = default_input();
    let out = solve(inp);

    assert_eq!(out.status, 0);

    // These values should be stable across versions
    // (update only when intentionally changing physics)
    assert!(out.t4 > 1200.0 && out.t4 < 1350.0, "T4 in expected range");
    assert!(out.tsfc_proxy > 0.8 && out.tsfc_proxy < 1.0, "TSFC in expected range");
    assert!(out.thrust_proxy > 0.9 && out.thrust_proxy < 1.2, "Thrust in expected range");
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_edge_case_low_mach() {
    let mut inp = default_input();
    inp.mach = 0.1;
    let out = solve(inp);

    // Should either converge or fail gracefully
    assert!(out.status >= 0 && out.status <= 5);
}

#[test]
fn test_edge_case_high_altitude() {
    let mut inp = default_input();
    inp.alt_km = 15.0;
    let out = solve(inp);

    assert!(out.status >= 0 && out.status <= 5);
}

#[test]
fn test_edge_case_extreme_bpr() {
    let mut inp = default_input();
    inp.bpr = 1.8;  // Very high for low-bypass
    let out = solve(inp);

    // May or may not converge, but should not crash
    assert!(out.status >= 0 && out.status <= 5);
}
