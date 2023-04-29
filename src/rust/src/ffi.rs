use std::os::raw::{c_double, c_int};

pub const MAX_HISTORY: usize = 256;

// Schema version (v2.1)
pub const SCHEMA_VERSION: &str = "1.0.0";
pub const SOLVER_VERSION: &str = "1.1.0";

// Component model identifiers (v2.4)
pub const MODEL_STANDARD: i32 = 0;
pub const MODEL_ADVANCED: i32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinotaurInput {
    pub mach: c_double,
    pub alt_km: c_double,
    pub bpr: c_double,
    pub opr: c_double,
    pub eta_comp: c_double,
    pub eta_turb: c_double,
    pub eta_nozz: c_double,
    pub fuel_k: c_double,
    pub max_iter: c_int,
    pub tol: c_double,
    pub damping: c_double,
    pub mass_tol: c_double,
    pub energy_tol: c_double,
    pub t4_max: c_double,
}

// Extended input with component models and degradation (v2.4)
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinotaurInputExt {
    // Base parameters (same as MinotaurInput)
    pub mach: c_double,
    pub alt_km: c_double,
    pub bpr: c_double,
    pub opr: c_double,
    pub eta_comp: c_double,
    pub eta_turb: c_double,
    pub eta_nozz: c_double,
    pub fuel_k: c_double,
    pub max_iter: c_int,
    pub tol: c_double,
    pub damping: c_double,
    pub mass_tol: c_double,
    pub energy_tol: c_double,
    pub t4_max: c_double,
    // Component model selection
    pub compressor_model: c_int,    // 0=standard, 1=advanced
    pub turbine_model: c_int,       // 0=standard, 1=advanced
    pub nozzle_model: c_int,        // 0=standard, 1=advanced
    // Loss coefficients
    pub inlet_loss: c_double,       // Inlet pressure loss coefficient
    pub burner_loss: c_double,      // Combustor pressure loss
    pub turbine_mech_loss: c_double, // Turbine mechanical loss
    pub nozzle_loss: c_double,      // Nozzle velocity loss coefficient
    // Degradation factors
    pub eta_comp_factor: c_double,  // Compressor efficiency multiplier (1.0 = nominal)
    pub eta_turb_factor: c_double,  // Turbine efficiency multiplier
    pub loss_adder: c_double,       // Additional pressure loss
    pub is_degraded: c_int,         // 1 if degradation scenario, 0 otherwise
}

impl MinotaurInputExt {
    /// Create extended input from base input with default component settings
    pub fn from_base(inp: &MinotaurInput) -> Self {
        Self {
            mach: inp.mach,
            alt_km: inp.alt_km,
            bpr: inp.bpr,
            opr: inp.opr,
            eta_comp: inp.eta_comp,
            eta_turb: inp.eta_turb,
            eta_nozz: inp.eta_nozz,
            fuel_k: inp.fuel_k,
            max_iter: inp.max_iter,
            tol: inp.tol,
            damping: inp.damping,
            mass_tol: inp.mass_tol,
            energy_tol: inp.energy_tol,
            t4_max: inp.t4_max,
            // Default component models (standard)
            compressor_model: MODEL_STANDARD,
            turbine_model: MODEL_STANDARD,
            nozzle_model: MODEL_STANDARD,
            // Default loss coefficients
            inlet_loss: 0.02,
            burner_loss: 0.04,
            turbine_mech_loss: 0.02,
            nozzle_loss: 0.01,
            // No degradation by default
            eta_comp_factor: 1.0,
            eta_turb_factor: 1.0,
            loss_adder: 0.0,
            is_degraded: 0,
        }
    }

    /// Create a degraded variant of this input
    pub fn with_degradation(
        &self,
        eta_comp_factor: f64,
        eta_turb_factor: f64,
        loss_adder: f64,
    ) -> Self {
        let mut degraded = *self;
        degraded.eta_comp_factor = eta_comp_factor;
        degraded.eta_turb_factor = eta_turb_factor;
        degraded.loss_adder = loss_adder;
        degraded.is_degraded = 1;
        degraded
    }

    /// Set component models
    pub fn with_models(
        &self,
        compressor: i32,
        turbine: i32,
        nozzle: i32,
    ) -> Self {
        let mut updated = *self;
        updated.compressor_model = compressor;
        updated.turbine_model = turbine;
        updated.nozzle_model = nozzle;
        updated
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinotaurOutput {
    pub status: c_int,
    pub iter: c_int,
    pub mass_resid: c_double,
    pub energy_resid: c_double,
    pub t4: c_double,
    pub tsfc_proxy: c_double,
    pub thrust_proxy: c_double,
    pub final_bpr: c_double,
    pub final_residual: c_double,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ConvergenceRecord {
    pub iteration: c_int,
    pub residual_norm: c_double,
    pub bpr: c_double,
    pub t4: c_double,
    pub step_size: c_double,
    pub admissible: c_int,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinotaurDiagnostics {
    pub history_len: c_int,
    pub history: [ConvergenceRecord; MAX_HISTORY],
    pub last_admissible_bpr: c_double,
    pub last_admissible_t4: c_double,
    pub line_search_steps: c_int,
    pub initial_residual: c_double,
    pub best_residual: c_double,
}

impl Default for MinotaurDiagnostics {
    fn default() -> Self {
        Self {
            history_len: 0,
            history: [ConvergenceRecord::default(); MAX_HISTORY],
            last_admissible_bpr: 0.0,
            last_admissible_t4: 0.0,
            line_search_steps: 0,
            initial_residual: 0.0,
            best_residual: 0.0,
        }
    }
}

// Comparison result for degradation analysis (v2.4)
#[derive(Clone, Debug)]
pub struct ComparisonResult {
    pub nominal: MinotaurOutput,
    pub degraded: MinotaurOutput,
    pub tsfc_change_pct: f64,
    pub thrust_change_pct: f64,
    pub t4_change_k: f64,
    pub iter_change: i32,
}

// Automatic Differentiation result (v2.8)
#[derive(Clone, Debug)]
pub struct ADResult {
    pub tsfc_val: f64,
    pub tsfc_der: f64,
    pub thrust_val: f64,
    pub thrust_der: f64,
    pub t4_val: f64,
    pub t4_der: f64,
    pub status: i32,
}

// Parameter indices for AD seeding (v2.8)
pub const SEED_MACH: i32 = 1;
pub const SEED_ALT: i32 = 2;
pub const SEED_BPR: i32 = 3;
pub const SEED_OPR: i32 = 4;
pub const SEED_ETA_COMP: i32 = 5;
pub const SEED_ETA_TURB: i32 = 6;

// Jacobian result (v2.8)
#[derive(Clone, Debug)]
pub struct JacobianResult {
    pub jacobian: [[f64; 3]; 6],  // 6 params x 3 outputs
    pub base_tsfc: f64,
    pub base_thrust: f64,
    pub base_t4: f64,
    pub status: i32,
    pub param_names: [&'static str; 6],
    pub output_names: [&'static str; 3],
}

impl JacobianResult {
    pub fn new() -> Self {
        Self {
            jacobian: [[0.0; 3]; 6],
            base_tsfc: 0.0,
            base_thrust: 0.0,
            base_t4: 0.0,
            status: 0,
            param_names: ["mach", "alt_km", "bpr", "opr", "eta_comp", "eta_turb"],
            output_names: ["tsfc", "thrust", "t4"],
        }
    }
}

extern "C" {
    pub fn minotaur_solve_c(inp: MinotaurInput, out: *mut MinotaurOutput);
    pub fn minotaur_solve_ext_c(inp_ext: MinotaurInputExt, out: *mut MinotaurOutput);
    pub fn minotaur_get_version(major: *mut c_int, minor: *mut c_int, patch: *mut c_int);
    pub fn minotaur_get_schema_version(major: *mut c_int, minor: *mut c_int, patch: *mut c_int);

    // Automatic Differentiation API (v2.8)
    pub fn minotaur_solve_ad_c(
        mach: c_double, alt_km: c_double, bpr: c_double, opr: c_double,
        eta_comp: c_double, eta_turb: c_double, eta_nozz: c_double,
        fuel_k: c_double, t4_max: c_double, seed_param: c_int,
        tsfc_val: *mut c_double, tsfc_der: *mut c_double,
        thrust_val: *mut c_double, thrust_der: *mut c_double,
        t4_val: *mut c_double, t4_der: *mut c_double,
        status: *mut c_int
    );

    pub fn minotaur_jacobian_c(
        mach: c_double, alt_km: c_double, bpr: c_double, opr: c_double,
        eta_comp: c_double, eta_turb: c_double, t4_max: c_double,
        jacobian: *mut [[c_double; 3]; 6],
        base_tsfc: *mut c_double, base_thrust: *mut c_double, base_t4: *mut c_double,
        status: *mut c_int
    );
}

pub fn solve(inp: MinotaurInput) -> MinotaurOutput {
    let mut out = MinotaurOutput {
        status: -999,
        iter: 0,
        mass_resid: 0.0,
        energy_resid: 0.0,
        t4: 0.0,
        tsfc_proxy: 0.0,
        thrust_proxy: 0.0,
        final_bpr: 0.0,
        final_residual: 0.0,
    };
    unsafe { minotaur_solve_c(inp, &mut out as *mut _) };
    out
}

/// Extended solve with component models and degradation (v2.4)
pub fn solve_ext(inp_ext: MinotaurInputExt) -> MinotaurOutput {
    let mut out = MinotaurOutput {
        status: -999,
        iter: 0,
        mass_resid: 0.0,
        energy_resid: 0.0,
        t4: 0.0,
        tsfc_proxy: 0.0,
        thrust_proxy: 0.0,
        final_bpr: 0.0,
        final_residual: 0.0,
    };
    unsafe { minotaur_solve_ext_c(inp_ext, &mut out as *mut _) };
    out
}

/// Compare nominal and degraded scenarios (v2.4)
pub fn compare_degradation(
    inp_ext: &MinotaurInputExt,
    eta_comp_factor: f64,
    eta_turb_factor: f64,
    loss_adder: f64,
) -> ComparisonResult {
    let nominal = solve_ext(*inp_ext);
    let degraded_inp = inp_ext.with_degradation(eta_comp_factor, eta_turb_factor, loss_adder);
    let degraded = solve_ext(degraded_inp);

    let tsfc_change_pct = if nominal.tsfc_proxy > 0.0 {
        (degraded.tsfc_proxy - nominal.tsfc_proxy) / nominal.tsfc_proxy * 100.0
    } else {
        0.0
    };

    let thrust_change_pct = if nominal.thrust_proxy > 0.0 {
        (degraded.thrust_proxy - nominal.thrust_proxy) / nominal.thrust_proxy * 100.0
    } else {
        0.0
    };

    ComparisonResult {
        nominal,
        degraded,
        tsfc_change_pct,
        thrust_change_pct,
        t4_change_k: degraded.t4 - nominal.t4,
        iter_change: degraded.iter - nominal.iter,
    }
}

/// Get solver version (v2.5)
pub fn get_solver_version() -> (i32, i32, i32) {
    let mut major: c_int = 0;
    let mut minor: c_int = 0;
    let mut patch: c_int = 0;
    unsafe {
        minotaur_get_version(&mut major, &mut minor, &mut patch);
    }
    (major, minor, patch)
}

/// Get schema version (v2.5)
pub fn get_schema_version() -> (i32, i32, i32) {
    let mut major: c_int = 0;
    let mut minor: c_int = 0;
    let mut patch: c_int = 0;
    unsafe {
        minotaur_get_schema_version(&mut major, &mut minor, &mut patch);
    }
    (major, minor, patch)
}

pub fn status_name(code: i32) -> &'static str {
    match code {
        0 => "OK",
        1 => "MAXITER",
        2 => "DIVERGED",
        3 => "INVARIANT_VIOL",
        4 => "CONSTRAINT_VIOL",
        5 => "NONPHYSICAL",
        _ => "UNKNOWN",
    }
}

/// Get component model name
pub fn model_name(code: i32) -> &'static str {
    match code {
        MODEL_STANDARD => "standard",
        MODEL_ADVANCED => "advanced",
        _ => "unknown",
    }
}

//-----------------------------------------------------------------------------
// Automatic Differentiation API (v2.8)
//-----------------------------------------------------------------------------

/// Solve with automatic differentiation for single parameter (v2.8)
pub fn solve_ad(
    mach: f64, alt_km: f64, bpr: f64, opr: f64,
    eta_comp: f64, eta_turb: f64, eta_nozz: f64,
    fuel_k: f64, t4_max: f64, seed_param: i32
) -> ADResult {
    let mut tsfc_val: c_double = 0.0;
    let mut tsfc_der: c_double = 0.0;
    let mut thrust_val: c_double = 0.0;
    let mut thrust_der: c_double = 0.0;
    let mut t4_val: c_double = 0.0;
    let mut t4_der: c_double = 0.0;
    let mut status: c_int = 0;

    unsafe {
        minotaur_solve_ad_c(
            mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz,
            fuel_k, t4_max, seed_param,
            &mut tsfc_val, &mut tsfc_der,
            &mut thrust_val, &mut thrust_der,
            &mut t4_val, &mut t4_der,
            &mut status
        );
    }

    ADResult {
        tsfc_val,
        tsfc_der,
        thrust_val,
        thrust_der,
        t4_val,
        t4_der,
        status,
    }
}

/// Compute full Jacobian matrix via forward-mode AD (v2.8)
pub fn compute_jacobian(
    mach: f64, alt_km: f64, bpr: f64, opr: f64,
    eta_comp: f64, eta_turb: f64, t4_max: f64
) -> JacobianResult {
    let mut result = JacobianResult::new();
    let mut status: c_int = 0;

    unsafe {
        minotaur_jacobian_c(
            mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max,
            &mut result.jacobian,
            &mut result.base_tsfc,
            &mut result.base_thrust,
            &mut result.base_t4,
            &mut status
        );
    }

    result.status = status;
    result
}

/// Get seed parameter name (v2.8)
pub fn seed_param_name(code: i32) -> &'static str {
    match code {
        SEED_MACH => "mach",
        SEED_ALT => "alt_km",
        SEED_BPR => "bpr",
        SEED_OPR => "opr",
        SEED_ETA_COMP => "eta_comp",
        SEED_ETA_TURB => "eta_turb",
        _ => "unknown",
    }
}
