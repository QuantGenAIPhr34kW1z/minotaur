use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::os::raw::{c_double, c_int};

// Version constants
const VERSION: &str = "0.6.0";
const SCHEMA_VERSION: &str = "1.0.0";

// FFI structures matching Fortran types
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct MinotaurInput {
    mach: c_double,
    alt_km: c_double,
    bpr: c_double,
    opr: c_double,
    eta_comp: c_double,
    eta_turb: c_double,
    eta_nozz: c_double,
    fuel_k: c_double,
    max_iter: c_int,
    tol: c_double,
    damping: c_double,
    mass_tol: c_double,
    energy_tol: c_double,
    t4_max: c_double,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct MinotaurOutput {
    status: c_int,
    iter: c_int,
    mass_resid: c_double,
    energy_resid: c_double,
    t4: c_double,
    tsfc_proxy: c_double,
    thrust_proxy: c_double,
    final_bpr: c_double,
    final_residual: c_double,
}

// Extended input for component models and degradation
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct MinotaurInputExt {
    // Base parameters
    mach: c_double,
    alt_km: c_double,
    bpr: c_double,
    opr: c_double,
    eta_comp: c_double,
    eta_turb: c_double,
    eta_nozz: c_double,
    fuel_k: c_double,
    max_iter: c_int,
    tol: c_double,
    damping: c_double,
    mass_tol: c_double,
    energy_tol: c_double,
    t4_max: c_double,
    // Component models
    compressor_model: c_int,
    turbine_model: c_int,
    nozzle_model: c_int,
    // Loss coefficients
    inlet_loss: c_double,
    burner_loss: c_double,
    turbine_mech_loss: c_double,
    nozzle_loss: c_double,
    // Degradation
    eta_comp_factor: c_double,
    eta_turb_factor: c_double,
    loss_adder: c_double,
    is_degraded: c_int,
}

extern "C" {
    fn minotaur_solve_c(inp: MinotaurInput, out: *mut MinotaurOutput);
    fn minotaur_solve_ext_c(inp_ext: MinotaurInputExt, out: *mut MinotaurOutput);
}

fn solve_internal(inp: MinotaurInput) -> MinotaurOutput {
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

fn solve_ext_internal(inp_ext: MinotaurInputExt) -> MinotaurOutput {
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

/// Python result class for solver output
#[pyclass]
#[derive(Clone)]
struct SolverResult {
    #[pyo3(get)]
    status: i32,
    #[pyo3(get)]
    status_name: String,
    #[pyo3(get)]
    converged: bool,
    #[pyo3(get)]
    iterations: i32,
    #[pyo3(get)]
    mass_residual: f64,
    #[pyo3(get)]
    energy_residual: f64,
    #[pyo3(get)]
    t4: f64,
    #[pyo3(get)]
    tsfc_proxy: f64,
    #[pyo3(get)]
    thrust_proxy: f64,
    #[pyo3(get)]
    final_bpr: f64,
    #[pyo3(get)]
    final_residual: f64,
}

#[pymethods]
impl SolverResult {
    fn __repr__(&self) -> String {
        format!(
            "SolverResult(status={}, converged={}, iter={}, t4={:.1}, tsfc={:.4}, thrust={:.4})",
            self.status, self.converged, self.iterations, self.t4, self.tsfc_proxy, self.thrust_proxy
        )
    }

    fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        dict.set_item("status", self.status)?;
        dict.set_item("status_name", &self.status_name)?;
        dict.set_item("converged", self.converged)?;
        dict.set_item("iterations", self.iterations)?;
        dict.set_item("mass_residual", self.mass_residual)?;
        dict.set_item("energy_residual", self.energy_residual)?;
        dict.set_item("t4", self.t4)?;
        dict.set_item("tsfc_proxy", self.tsfc_proxy)?;
        dict.set_item("thrust_proxy", self.thrust_proxy)?;
        dict.set_item("final_bpr", self.final_bpr)?;
        dict.set_item("final_residual", self.final_residual)?;
        Ok(dict.into())
    }
}

fn status_name(code: i32) -> &'static str {
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

fn output_to_result(out: &MinotaurOutput) -> SolverResult {
    SolverResult {
        status: out.status,
        status_name: status_name(out.status).to_string(),
        converged: out.status == 0,
        iterations: out.iter,
        mass_residual: out.mass_resid,
        energy_residual: out.energy_resid,
        t4: out.t4,
        tsfc_proxy: out.tsfc_proxy,
        thrust_proxy: out.thrust_proxy,
        final_bpr: out.final_bpr,
        final_residual: out.final_residual,
    }
}

/// Solve a single turbofan cycle point
#[pyfunction]
#[pyo3(signature = (
    mach,
    alt_km,
    bpr,
    opr,
    eta_comp = 0.82,
    eta_turb = 0.86,
    eta_nozz = 0.95,
    fuel_k = 1.0,
    max_iter = 200,
    tol = 1e-10,
    damping = 0.5,
    mass_tol = 1e-9,
    energy_tol = 1e-9,
    t4_max = 1400.0
))]
fn solve(
    mach: f64,
    alt_km: f64,
    bpr: f64,
    opr: f64,
    eta_comp: f64,
    eta_turb: f64,
    eta_nozz: f64,
    fuel_k: f64,
    max_iter: i32,
    tol: f64,
    damping: f64,
    mass_tol: f64,
    energy_tol: f64,
    t4_max: f64,
) -> PyResult<SolverResult> {
    let inp = MinotaurInput {
        mach,
        alt_km,
        bpr,
        opr,
        eta_comp,
        eta_turb,
        eta_nozz,
        fuel_k,
        max_iter,
        tol,
        damping,
        mass_tol,
        energy_tol,
        t4_max,
    };

    let out = solve_internal(inp);
    Ok(output_to_result(&out))
}

/// Solve with extended options (component models, losses, degradation)
#[pyfunction]
#[pyo3(signature = (
    mach,
    alt_km,
    bpr,
    opr,
    eta_comp = 0.82,
    eta_turb = 0.86,
    eta_nozz = 0.95,
    fuel_k = 1.0,
    max_iter = 200,
    tol = 1e-10,
    damping = 0.5,
    mass_tol = 1e-9,
    energy_tol = 1e-9,
    t4_max = 1400.0,
    compressor_model = 0,
    turbine_model = 0,
    nozzle_model = 0,
    inlet_loss = 0.02,
    burner_loss = 0.04,
    turbine_mech_loss = 0.02,
    nozzle_loss = 0.01,
    eta_comp_factor = 1.0,
    eta_turb_factor = 1.0,
    loss_adder = 0.0
))]
fn solve_extended(
    mach: f64,
    alt_km: f64,
    bpr: f64,
    opr: f64,
    eta_comp: f64,
    eta_turb: f64,
    eta_nozz: f64,
    fuel_k: f64,
    max_iter: i32,
    tol: f64,
    damping: f64,
    mass_tol: f64,
    energy_tol: f64,
    t4_max: f64,
    compressor_model: i32,
    turbine_model: i32,
    nozzle_model: i32,
    inlet_loss: f64,
    burner_loss: f64,
    turbine_mech_loss: f64,
    nozzle_loss: f64,
    eta_comp_factor: f64,
    eta_turb_factor: f64,
    loss_adder: f64,
) -> PyResult<SolverResult> {
    let is_degraded = if eta_comp_factor < 1.0 || eta_turb_factor < 1.0 || loss_adder > 0.0 {
        1
    } else {
        0
    };

    let inp_ext = MinotaurInputExt {
        mach,
        alt_km,
        bpr,
        opr,
        eta_comp,
        eta_turb,
        eta_nozz,
        fuel_k,
        max_iter,
        tol,
        damping,
        mass_tol,
        energy_tol,
        t4_max,
        compressor_model,
        turbine_model,
        nozzle_model,
        inlet_loss,
        burner_loss,
        turbine_mech_loss,
        nozzle_loss,
        eta_comp_factor,
        eta_turb_factor,
        loss_adder,
        is_degraded,
    };

    let out = solve_ext_internal(inp_ext);
    Ok(output_to_result(&out))
}

/// Run a parameter sweep over BPR and OPR ranges (NumPy-compatible)
#[pyfunction]
fn sweep<'py>(
    py: Python<'py>,
    bpr_values: PyReadonlyArray1<'py, f64>,
    opr_values: PyReadonlyArray1<'py, f64>,
    mach: f64,
    alt_km: f64,
    eta_comp: f64,
    eta_turb: f64,
    eta_nozz: f64,
    t4_max: f64,
) -> PyResult<&'py PyDict> {
    let bpr_arr = bpr_values.as_slice()?;
    let opr_arr = opr_values.as_slice()?;

    let n_bpr = bpr_arr.len();
    let n_opr = opr_arr.len();
    let n_total = n_bpr * n_opr;

    let mut status_vec = Vec::with_capacity(n_total);
    let mut iter_vec = Vec::with_capacity(n_total);
    let mut t4_vec = Vec::with_capacity(n_total);
    let mut tsfc_vec = Vec::with_capacity(n_total);
    let mut thrust_vec = Vec::with_capacity(n_total);
    let mut bpr_out = Vec::with_capacity(n_total);
    let mut opr_out = Vec::with_capacity(n_total);

    for &bpr in bpr_arr {
        for &opr in opr_arr {
            let inp = MinotaurInput {
                mach,
                alt_km,
                bpr,
                opr,
                eta_comp,
                eta_turb,
                eta_nozz,
                fuel_k: 1.0,
                max_iter: 200,
                tol: 1e-10,
                damping: 0.5,
                mass_tol: 1e-9,
                energy_tol: 1e-9,
                t4_max,
            };

            let out = solve_internal(inp);

            status_vec.push(out.status);
            iter_vec.push(out.iter);
            t4_vec.push(out.t4);
            tsfc_vec.push(out.tsfc_proxy);
            thrust_vec.push(out.thrust_proxy);
            bpr_out.push(bpr);
            opr_out.push(opr);
        }
    }

    let result = PyDict::new(py);
    result.set_item("bpr", PyArray1::from_vec(py, bpr_out))?;
    result.set_item("opr", PyArray1::from_vec(py, opr_out))?;
    result.set_item("status", PyArray1::from_vec(py, status_vec))?;
    result.set_item("iterations", PyArray1::from_vec(py, iter_vec))?;
    result.set_item("t4", PyArray1::from_vec(py, t4_vec))?;
    result.set_item("tsfc", PyArray1::from_vec(py, tsfc_vec))?;
    result.set_item("thrust", PyArray1::from_vec(py, thrust_vec))?;
    result.set_item("n_bpr", n_bpr)?;
    result.set_item("n_opr", n_opr)?;

    Ok(result)
}

/// Compute local sensitivities via central finite differences
#[pyfunction]
#[pyo3(signature = (
    mach,
    alt_km,
    bpr,
    opr,
    eta_comp = 0.82,
    eta_turb = 0.86,
    step = 1e-6
))]
fn sensitivity<'py>(
    py: Python<'py>,
    mach: f64,
    alt_km: f64,
    bpr: f64,
    opr: f64,
    eta_comp: f64,
    eta_turb: f64,
    step: f64,
) -> PyResult<&'py PyDict> {
    let base_inp = MinotaurInput {
        mach,
        alt_km,
        bpr,
        opr,
        eta_comp,
        eta_turb,
        eta_nozz: 0.95,
        fuel_k: 1.0,
        max_iter: 200,
        tol: 1e-10,
        damping: 0.5,
        mass_tol: 1e-9,
        energy_tol: 1e-9,
        t4_max: 1400.0,
    };

    let base_out = solve_internal(base_inp);

    // Parameter perturbations
    let params = ["bpr", "opr", "eta_comp", "eta_turb", "mach", "alt_km"];
    let mut jacobian = Vec::new();

    for param in &params {
        let (h, mut inp_plus, mut inp_minus) = match *param {
            "bpr" => {
                let h = bpr * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.bpr = bpr + h;
                m.bpr = bpr - h;
                (h, p, m)
            }
            "opr" => {
                let h = opr * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.opr = opr + h;
                m.opr = opr - h;
                (h, p, m)
            }
            "eta_comp" => {
                let h = eta_comp * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.eta_comp = eta_comp + h;
                m.eta_comp = eta_comp - h;
                (h, p, m)
            }
            "eta_turb" => {
                let h = eta_turb * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.eta_turb = eta_turb + h;
                m.eta_turb = eta_turb - h;
                (h, p, m)
            }
            "mach" => {
                let h = mach * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.mach = mach + h;
                m.mach = mach - h;
                (h, p, m)
            }
            "alt_km" => {
                let h = alt_km.max(1.0) * step;
                let mut p = base_inp;
                let mut m = base_inp;
                p.alt_km = alt_km + h;
                m.alt_km = alt_km - h;
                (h, p, m)
            }
            _ => continue,
        };

        let out_plus = solve_internal(inp_plus);
        let out_minus = solve_internal(inp_minus);

        let two_h = 2.0 * h;
        jacobian.push(vec![
            (out_plus.tsfc_proxy - out_minus.tsfc_proxy) / two_h,
            (out_plus.thrust_proxy - out_minus.thrust_proxy) / two_h,
            (out_plus.t4 - out_minus.t4) / two_h,
        ]);
    }

    let result = PyDict::new(py);
    result.set_item("parameters", params.to_vec())?;
    result.set_item("outputs", vec!["tsfc", "thrust", "t4"])?;

    // Convert jacobian to 2D numpy array
    let flat: Vec<f64> = jacobian.iter().flatten().copied().collect();
    let arr = PyArray2::from_vec(py, flat)
        .reshape([6, 3])
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("{:?}", e)))?;
    result.set_item("jacobian", arr)?;

    result.set_item("base_tsfc", base_out.tsfc_proxy)?;
    result.set_item("base_thrust", base_out.thrust_proxy)?;
    result.set_item("base_t4", base_out.t4)?;

    Ok(result)
}

/// Compare nominal vs degraded performance
#[pyfunction]
#[pyo3(signature = (
    mach,
    alt_km,
    bpr,
    opr,
    eta_comp = 0.82,
    eta_turb = 0.86,
    degradation_level = "moderate"
))]
fn compare_degradation(
    mach: f64,
    alt_km: f64,
    bpr: f64,
    opr: f64,
    eta_comp: f64,
    eta_turb: f64,
    degradation_level: &str,
) -> PyResult<(SolverResult, SolverResult, f64, f64, f64)> {
    let (kc, kt, delta_loss) = match degradation_level {
        "light" => (0.95, 0.97, 0.01),
        "moderate" => (0.90, 0.94, 0.02),
        "severe" => (0.85, 0.91, 0.03),
        _ => {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "degradation_level must be 'light', 'moderate', or 'severe'",
            ))
        }
    };

    // Nominal run
    let inp_nom = MinotaurInput {
        mach,
        alt_km,
        bpr,
        opr,
        eta_comp,
        eta_turb,
        eta_nozz: 0.95,
        fuel_k: 1.0,
        max_iter: 200,
        tol: 1e-10,
        damping: 0.5,
        mass_tol: 1e-9,
        energy_tol: 1e-9,
        t4_max: 1400.0,
    };
    let out_nom = solve_internal(inp_nom);

    // Degraded run
    let inp_deg = MinotaurInputExt {
        mach,
        alt_km,
        bpr,
        opr,
        eta_comp,
        eta_turb,
        eta_nozz: 0.95,
        fuel_k: 1.0,
        max_iter: 200,
        tol: 1e-10,
        damping: 0.5,
        mass_tol: 1e-9,
        energy_tol: 1e-9,
        t4_max: 1400.0,
        compressor_model: 0,
        turbine_model: 0,
        nozzle_model: 0,
        inlet_loss: 0.02,
        burner_loss: 0.04,
        turbine_mech_loss: 0.02,
        nozzle_loss: 0.01,
        eta_comp_factor: kc,
        eta_turb_factor: kt,
        loss_adder: delta_loss,
        is_degraded: 1,
    };
    let out_deg = solve_ext_internal(inp_deg);

    let tsfc_change_pct = if out_nom.tsfc_proxy > 0.0 {
        (out_deg.tsfc_proxy - out_nom.tsfc_proxy) / out_nom.tsfc_proxy * 100.0
    } else {
        0.0
    };

    let thrust_change_pct = if out_nom.thrust_proxy > 0.0 {
        (out_deg.thrust_proxy - out_nom.thrust_proxy) / out_nom.thrust_proxy * 100.0
    } else {
        0.0
    };

    let t4_change = out_deg.t4 - out_nom.t4;

    Ok((
        output_to_result(&out_nom),
        output_to_result(&out_deg),
        tsfc_change_pct,
        thrust_change_pct,
        t4_change,
    ))
}

/// Get version information
#[pyfunction]
fn version() -> PyResult<(String, String)> {
    Ok((VERSION.to_string(), SCHEMA_VERSION.to_string()))
}

/// MINOTAUR Python module
#[pymodule]
fn minotaur_python(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<SolverResult>()?;
    m.add_function(wrap_pyfunction!(solve, m)?)?;
    m.add_function(wrap_pyfunction!(solve_extended, m)?)?;
    m.add_function(wrap_pyfunction!(sweep, m)?)?;
    m.add_function(wrap_pyfunction!(sensitivity, m)?)?;
    m.add_function(wrap_pyfunction!(compare_degradation, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    // Constants
    m.add("__version__", VERSION)?;
    m.add("SCHEMA_VERSION", SCHEMA_VERSION)?;
    m.add("STATUS_OK", 0)?;
    m.add("STATUS_MAXITER", 1)?;
    m.add("STATUS_DIVERGED", 2)?;
    m.add("STATUS_INVARIANT_VIOL", 3)?;
    m.add("STATUS_CONSTRAINT_VIOL", 4)?;
    m.add("STATUS_NONPHYSICAL", 5)?;

    Ok(())
}
