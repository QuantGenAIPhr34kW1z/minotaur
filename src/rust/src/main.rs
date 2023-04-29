mod config;
mod ffi;
mod io;
mod nsga2;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const VERSION: &str = "1.1.0";
const SCHEMA_VERSION: &str = "1.0.0";
const CSTNSystems_PROGRAM_ID: &str = "CSTNSystems-MINOTAUR";

#[derive(Parser, Debug)]
#[command(name = "minotaur")]
#[command(author = "CSTNSystems")]
#[command(version)]
#[command(about = "CSTNSystems/MINOTAUR - Deterministic reduced-order turbofan cycle solver")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to TOML configuration file
    #[arg(short, long, global = true)]
    config: Option<String>,

    /// Output path (file or directory)
    #[arg(short, long, global = true)]
    out: Option<String>,

    /// Run mode: "single" or "sweep" (legacy compatibility)
    #[arg(short, long, default_value = "single")]
    mode: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a single configuration
    Run {
        /// Generate JSON manifest and summary
        #[arg(long)]
        json: bool,
    },
    /// Run a parameter sweep
    Sweep {
        /// Generate JSON outputs
        #[arg(long)]
        json: bool,
    },
    /// Compute local sensitivities via finite differences
    Sensitivity {
        /// Relative step size for finite differences
        #[arg(long, default_value = "1e-6")]
        step: f64,
    },
    /// Validate a configuration file
    Validate,
    /// Compare nominal vs degraded scenarios (v2.4)
    Compare {
        /// Degradation level: light, moderate, severe, or custom
        #[arg(long, default_value = "moderate")]
        level: String,
        /// Generate JSON outputs
        #[arg(long)]
        json: bool,
    },
    /// Print version information (v2.5)
    Version,
    /// Compute exact Jacobian via forward-mode AD (v2.8)
    Jacobian {
        /// Generate JSON outputs
        #[arg(long)]
        json: bool,
    },
    /// Run multi-objective optimization via NSGA-II (v2.9)
    Optimize {
        /// Population size
        #[arg(long, default_value = "100")]
        pop_size: usize,
        /// Number of generations
        #[arg(long, default_value = "50")]
        generations: usize,
        /// Random seed for reproducibility
        #[arg(long, default_value = "42")]
        seed: u64,
        /// Generate JSON outputs
        #[arg(long)]
        json: bool,
    },
}

// ============================================================================
// JSON Output Structures (v2.5 result bundles with schema v2.0.0)
// ============================================================================

#[derive(Serialize)]
struct Manifest {
    schema_version: String,
    solver_version: String,
    CSTNSystems_program_id: String,
    timestamp_utc: String,
    git_commit: Option<String>,
    git_dirty: bool,
    platform: String,
    rust_version: String,
    config_hash: String,
    config_snapshot: config::Root,
}

#[derive(Serialize)]
struct Summary {
    status: i32,
    status_name: String,
    converged: bool,
    iterations: i32,
    final_residual: f64,
    mass_residual: f64,
    energy_residual: f64,
    t4: f64,
    tsfc_proxy: f64,
    thrust_proxy: f64,
    wall_time_ms: f64,
}

#[derive(Serialize)]
struct ConvergenceHistory {
    iteration: i32,
    residual_norm: f64,
    bpr: f64,
    t4: f64,
    step_size: f64,
    admissible: bool,
}

#[derive(Serialize)]
struct Convergence {
    history: Vec<ConvergenceHistory>,
    final_state: FinalState,
}

#[derive(Serialize)]
struct FinalState {
    bpr: f64,
    opr: f64,
    t4: f64,
}

#[derive(Serialize)]
struct SensitivityOutput {
    parameters: Vec<String>,
    outputs: Vec<String>,
    jacobian: Vec<Vec<f64>>,
    step_sizes: HashMap<String, f64>,
    base_values: HashMap<String, f64>,
}

// v2.8: Jacobian output via AD
#[derive(Serialize)]
struct JacobianOutput {
    manifest: Manifest,
    method: String,
    parameters: Vec<String>,
    outputs: Vec<String>,
    jacobian: Vec<Vec<f64>>,
    base_values: HashMap<String, f64>,
    status: i32,
}

// v2.9: Optimization output structures
#[derive(Serialize)]
struct OptimizationOutput {
    manifest: Manifest,
    config: OptConfig,
    pareto_front: Vec<ParetoSolution>,
    hypervolume: Option<f64>,
    generations: usize,
    wall_time_ms: f64,
}

#[derive(Serialize)]
struct OptConfig {
    pop_size: usize,
    generations: usize,
    crossover_prob: f64,
    mutation_prob: f64,
    seed: u64,
    bounds: Vec<(f64, f64)>,
    objectives: Vec<String>,
}

#[derive(Serialize)]
struct ParetoSolution {
    bpr: f64,
    opr: f64,
    eta_comp: f64,
    eta_turb: f64,
    tsfc: f64,
    thrust: f64,
    t4: f64,
    status: i32,
    rank: usize,
    crowding_distance: f64,
}

#[derive(Serialize)]
struct ResultBundle {
    manifest: Manifest,
    summary: Summary,
    convergence: Option<Convergence>,
}

// v2.4: Comparison output structures
#[derive(Serialize)]
struct ComparisonOutput {
    manifest: Manifest,
    nominal: ScenarioResult,
    degraded: ScenarioResult,
    delta: DeltaMetrics,
}

#[derive(Serialize)]
struct ScenarioResult {
    scenario_name: String,
    status: i32,
    status_name: String,
    converged: bool,
    iterations: i32,
    t4: f64,
    tsfc_proxy: f64,
    thrust_proxy: f64,
    eta_comp_effective: f64,
    eta_turb_effective: f64,
}

#[derive(Serialize)]
struct DeltaMetrics {
    tsfc_change_pct: f64,
    thrust_change_pct: f64,
    t4_change_k: f64,
    iter_change: i32,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn compute_hash(data: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}{:016x}{:016x}{:016x}",
            hasher.finish(), hasher.finish(), hasher.finish(), hasher.finish())
}

fn get_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let secs_per_day = 86400u64;
    let days_since_epoch = now / secs_per_day;
    let secs_today = now % secs_per_day;
    let hours = secs_today / 3600;
    let mins = (secs_today % 3600) / 60;
    let secs = secs_today % 60;

    let mut year = 1970u64;
    let mut remaining_days = days_since_epoch;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month = 1u64;
    for &days in &month_days {
        let d = if month == 2 && year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { days };
        if remaining_days < d {
            break;
        }
        remaining_days -= d;
        month += 1;
    }
    let day = remaining_days + 1;

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hours, mins, secs)
}

fn create_input(cfg: &config::Root, bpr: f64, opr: f64) -> ffi::MinotaurInput {
    ffi::MinotaurInput {
        mach: cfg.cycle.mach,
        alt_km: cfg.cycle.alt_km,
        bpr,
        opr,
        eta_comp: cfg.cycle.eta_comp,
        eta_turb: cfg.cycle.eta_turb,
        eta_nozz: cfg.cycle.eta_nozz,
        fuel_k: cfg.cycle.fuel_k,
        max_iter: cfg.solver.max_iter,
        tol: cfg.solver.tol,
        damping: cfg.solver.damping,
        mass_tol: cfg.invariants.mass_tol,
        energy_tol: cfg.invariants.energy_tol,
        t4_max: cfg.constraints.t4_max,
    }
}

// v2.4: Create extended input with component models and losses
fn create_input_ext(cfg: &config::Root, bpr: f64, opr: f64) -> ffi::MinotaurInputExt {
    let base = create_input(cfg, bpr, opr);
    let mut ext = ffi::MinotaurInputExt::from_base(&base);

    // Apply component model selection
    if let Some(ref comp) = cfg.components {
        ext.compressor_model = comp.compressor_id();
        ext.turbine_model = comp.turbine_id();
        ext.nozzle_model = comp.nozzle_id();
    }

    // Apply loss coefficients
    if let Some(ref losses) = cfg.losses {
        ext.inlet_loss = losses.inlet;
        ext.burner_loss = losses.burner;
        ext.turbine_mech_loss = losses.turbine;
        ext.nozzle_loss = losses.nozzle;
    }

    // Apply degradation if specified
    if let Some(ref deg) = cfg.degradation {
        ext.eta_comp_factor = deg.eta_comp_factor;
        ext.eta_turb_factor = deg.eta_turb_factor;
        ext.loss_adder = deg.loss_adder;
        ext.is_degraded = if deg.is_degraded() { 1 } else { 0 };
    }

    ext
}

fn create_manifest(cfg: &config::Root, cfg_text: &str) -> Manifest {
    Manifest {
        schema_version: SCHEMA_VERSION.to_string(),
        solver_version: VERSION.to_string(),
        CSTNSystems_program_id: CSTNSystems_PROGRAM_ID.to_string(),
        timestamp_utc: get_timestamp(),
        git_commit: None,
        git_dirty: false,
        platform: std::env::consts::OS.to_string(),
        rust_version: "stable".to_string(),
        config_hash: compute_hash(cfg_text),
        config_snapshot: cfg.clone(),
    }
}

fn create_summary(out: &ffi::MinotaurOutput, wall_time_ms: f64) -> Summary {
    Summary {
        status: out.status,
        status_name: ffi::status_name(out.status).to_string(),
        converged: out.status == 0,
        iterations: out.iter,
        final_residual: out.final_residual,
        mass_residual: out.mass_resid,
        energy_residual: out.energy_resid,
        t4: out.t4,
        tsfc_proxy: out.tsfc_proxy,
        thrust_proxy: out.thrust_proxy,
        wall_time_ms,
    }
}

// ============================================================================
// Run Modes
// ============================================================================

fn run_single(cfg: &config::Root, cfg_text: &str, out_path: &str, json_output: bool) -> Result<()> {
    let bpr = cfg.cycle.bpr.context("cycle.bpr required for single mode")?;
    let opr = cfg.cycle.opr.context("cycle.opr required for single mode")?;

    let inp = create_input(cfg, bpr, opr);

    let start = Instant::now();
    let out = ffi::solve(inp);
    let wall_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // CSV output
    let mut w = io::CsvWriter::create(out_path)?;
    w.write_header()?;
    w.write_row("baseline", bpr, opr, cfg.cycle.mach, cfg.cycle.alt_km, &out)?;
    w.flush()?;

    eprintln!(
        "[minotaur] status={} ({}) iter={} residual={:.2e} t4={:.1} tsfc={:.4} thrust={:.4}",
        out.status,
        ffi::status_name(out.status),
        out.iter,
        out.final_residual,
        out.t4,
        out.tsfc_proxy,
        out.thrust_proxy
    );

    // JSON output (v2.5 result bundle with schema v2.0.0)
    if json_output {
        let json_path = out_path.replace(".csv", ".json");
        let bundle = ResultBundle {
            manifest: create_manifest(cfg, cfg_text),
            summary: create_summary(&out, wall_time_ms),
            convergence: None,
        };
        let json = serde_json::to_string_pretty(&bundle)?;
        fs::write(&json_path, json)?;
        eprintln!("[minotaur] JSON bundle: {}", json_path);
    }

    if out.status != 0 {
        eprintln!("[minotaur] WARNING: solver did not converge");
    }

    Ok(())
}

fn run_sweep(cfg: &config::Root, cfg_text: &str, out_path: &str, json_output: bool) -> Result<()> {
    let sweep = cfg.sweep.as_ref().context("[sweep] section required for sweep mode")?;

    let mut w = io::CsvWriter::create(out_path)?;
    w.write_header()?;

    let mut total = 0;
    let mut converged = 0;
    let mut results: Vec<(String, f64, f64, ffi::MinotaurOutput)> = Vec::new();

    let start = Instant::now();

    for i in 0..sweep.bpr_n {
        let bpr = if sweep.bpr_n > 1 {
            sweep.bpr_min + (sweep.bpr_max - sweep.bpr_min) * (i as f64) / ((sweep.bpr_n - 1) as f64)
        } else {
            sweep.bpr_min
        };

        for j in 0..sweep.opr_n {
            let opr = if sweep.opr_n > 1 {
                sweep.opr_min + (sweep.opr_max - sweep.opr_min) * (j as f64) / ((sweep.opr_n - 1) as f64)
            } else {
                sweep.opr_min
            };

            let inp = create_input(cfg, bpr, opr);
            let out = ffi::solve(inp);
            let case = format!("sweep_{:04}_{:04}", i, j);
            w.write_row(&case, bpr, opr, cfg.cycle.mach, cfg.cycle.alt_km, &out)?;

            if json_output {
                results.push((case, bpr, opr, out));
            }

            total += 1;
            if out.status == 0 {
                converged += 1;
            }
        }
    }

    w.flush()?;
    let wall_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    eprintln!(
        "[minotaur] sweep complete: {}/{} converged ({:.1}%) in {:.1}ms",
        converged,
        total,
        100.0 * converged as f64 / total as f64,
        wall_time_ms
    );

    if json_output {
        let json_path = out_path.replace(".csv", "_summary.json");

        #[derive(Serialize)]
        struct SweepSummary {
            manifest: Manifest,
            total_runs: usize,
            converged_runs: usize,
            convergence_rate: f64,
            wall_time_ms: f64,
            parameter_ranges: ParameterRanges,
        }

        #[derive(Serialize)]
        struct ParameterRanges {
            bpr: (f64, f64, usize),
            opr: (f64, f64, usize),
        }

        let summary = SweepSummary {
            manifest: create_manifest(cfg, cfg_text),
            total_runs: total,
            converged_runs: converged,
            convergence_rate: converged as f64 / total as f64,
            wall_time_ms,
            parameter_ranges: ParameterRanges {
                bpr: (sweep.bpr_min, sweep.bpr_max, sweep.bpr_n),
                opr: (sweep.opr_min, sweep.opr_max, sweep.opr_n),
            },
        };

        let json = serde_json::to_string_pretty(&summary)?;
        fs::write(&json_path, json)?;
        eprintln!("[minotaur] JSON summary: {}", json_path);
    }

    Ok(())
}

fn run_sensitivity(cfg: &config::Root, out_path: &str, step: f64) -> Result<()> {
    let bpr = cfg.cycle.bpr.context("cycle.bpr required")?;
    let opr = cfg.cycle.opr.context("cycle.opr required")?;

    let params = ["bpr", "opr", "eta_comp", "eta_turb", "mach", "alt_km"];
    let outputs = ["tsfc_proxy", "thrust_proxy", "t4", "iterations"];

    let inp_base = create_input(cfg, bpr, opr);
    let out_base = ffi::solve(inp_base);

    if out_base.status != 0 {
        anyhow::bail!("Base configuration does not converge (status={})", out_base.status);
    }

    let mut jacobian: Vec<Vec<f64>> = Vec::new();
    let mut step_sizes: HashMap<String, f64> = HashMap::new();
    let mut base_values: HashMap<String, f64> = HashMap::new();

    base_values.insert("tsfc_proxy".to_string(), out_base.tsfc_proxy);
    base_values.insert("thrust_proxy".to_string(), out_base.thrust_proxy);
    base_values.insert("t4".to_string(), out_base.t4);
    base_values.insert("iterations".to_string(), out_base.iter as f64);

    for param in &params {
        let (val, perturbed_inp_plus, perturbed_inp_minus) = match *param {
            "bpr" => {
                let h = bpr * step;
                step_sizes.insert(param.to_string(), h);
                (bpr, create_input(cfg, bpr + h, opr), create_input(cfg, bpr - h, opr))
            }
            "opr" => {
                let h = opr * step;
                step_sizes.insert(param.to_string(), h);
                (opr, create_input(cfg, bpr, opr + h), create_input(cfg, bpr, opr - h))
            }
            "eta_comp" => {
                let h = cfg.cycle.eta_comp * step;
                step_sizes.insert(param.to_string(), h);
                let mut cfg_plus = cfg.clone();
                let mut cfg_minus = cfg.clone();
                cfg_plus.cycle.eta_comp += h;
                cfg_minus.cycle.eta_comp -= h;
                (cfg.cycle.eta_comp, create_input(&cfg_plus, bpr, opr), create_input(&cfg_minus, bpr, opr))
            }
            "eta_turb" => {
                let h = cfg.cycle.eta_turb * step;
                step_sizes.insert(param.to_string(), h);
                let mut cfg_plus = cfg.clone();
                let mut cfg_minus = cfg.clone();
                cfg_plus.cycle.eta_turb += h;
                cfg_minus.cycle.eta_turb -= h;
                (cfg.cycle.eta_turb, create_input(&cfg_plus, bpr, opr), create_input(&cfg_minus, bpr, opr))
            }
            "mach" => {
                let h = cfg.cycle.mach * step;
                step_sizes.insert(param.to_string(), h);
                let mut cfg_plus = cfg.clone();
                let mut cfg_minus = cfg.clone();
                cfg_plus.cycle.mach += h;
                cfg_minus.cycle.mach -= h;
                (cfg.cycle.mach, create_input(&cfg_plus, bpr, opr), create_input(&cfg_minus, bpr, opr))
            }
            "alt_km" => {
                let h = cfg.cycle.alt_km.max(1.0) * step;
                step_sizes.insert(param.to_string(), h);
                let mut cfg_plus = cfg.clone();
                let mut cfg_minus = cfg.clone();
                cfg_plus.cycle.alt_km += h;
                cfg_minus.cycle.alt_km -= h;
                (cfg.cycle.alt_km, create_input(&cfg_plus, bpr, opr), create_input(&cfg_minus, bpr, opr))
            }
            _ => continue,
        };

        let out_plus = ffi::solve(perturbed_inp_plus);
        let out_minus = ffi::solve(perturbed_inp_minus);

        let h = step_sizes[*param];
        let two_h = 2.0 * h;

        let mut row = Vec::new();
        row.push((out_plus.tsfc_proxy - out_minus.tsfc_proxy) / two_h);
        row.push((out_plus.thrust_proxy - out_minus.thrust_proxy) / two_h);
        row.push((out_plus.t4 - out_minus.t4) / two_h);
        row.push((out_plus.iter as f64 - out_minus.iter as f64) / two_h);
        jacobian.push(row);

        base_values.insert(param.to_string(), val);
    }

    let mut file = fs::File::create(out_path)?;
    writeln!(file, "parameter,tsfc_proxy,thrust_proxy,t4,iterations")?;
    for (i, param) in params.iter().enumerate() {
        writeln!(file, "{},{:.6e},{:.6e},{:.6e},{:.6e}",
                 param, jacobian[i][0], jacobian[i][1], jacobian[i][2], jacobian[i][3])?;
    }

    let json_path = out_path.replace(".csv", ".json");
    let sens = SensitivityOutput {
        parameters: params.iter().map(|s| s.to_string()).collect(),
        outputs: outputs.iter().map(|s| s.to_string()).collect(),
        jacobian,
        step_sizes,
        base_values,
    };
    let json = serde_json::to_string_pretty(&sens)?;
    fs::write(&json_path, json)?;

    eprintln!("[minotaur] sensitivity analysis complete");
    eprintln!("[minotaur] CSV: {}", out_path);
    eprintln!("[minotaur] JSON: {}", json_path);

    Ok(())
}

// v2.4: Compare nominal vs degraded scenarios
fn run_compare(cfg: &config::Root, cfg_text: &str, out_path: &str, level: &str, json_output: bool) -> Result<()> {
    let bpr = cfg.cycle.bpr.context("cycle.bpr required")?;
    let opr = cfg.cycle.opr.context("cycle.opr required")?;

    // Get degradation parameters based on level
    let degradation = match level {
        "light" => config::Degradation::light(),
        "moderate" => config::Degradation::moderate(),
        "severe" => config::Degradation::severe(),
        "custom" => cfg.degradation.clone().unwrap_or_default(),
        _ => anyhow::bail!("Unknown degradation level: {}. Use light, moderate, severe, or custom", level),
    };

    // Create nominal input
    let inp_nominal = create_input_ext(cfg, bpr, opr);

    // Create degraded input
    let inp_degraded = inp_nominal.with_degradation(
        degradation.eta_comp_factor,
        degradation.eta_turb_factor,
        degradation.loss_adder,
    );

    let start = Instant::now();
    let out_nominal = ffi::solve_ext(inp_nominal);
    let out_degraded = ffi::solve_ext(inp_degraded);
    let wall_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Calculate deltas
    let tsfc_change_pct = if out_nominal.tsfc_proxy > 0.0 {
        (out_degraded.tsfc_proxy - out_nominal.tsfc_proxy) / out_nominal.tsfc_proxy * 100.0
    } else {
        0.0
    };

    let thrust_change_pct = if out_nominal.thrust_proxy > 0.0 {
        (out_degraded.thrust_proxy - out_nominal.thrust_proxy) / out_nominal.thrust_proxy * 100.0
    } else {
        0.0
    };

    let t4_change_k = out_degraded.t4 - out_nominal.t4;
    let iter_change = out_degraded.iter - out_nominal.iter;

    // Write CSV
    let mut file = fs::File::create(out_path)?;
    writeln!(file, "scenario,status,converged,iter,t4,tsfc_proxy,thrust_proxy,eta_comp_eff,eta_turb_eff")?;
    writeln!(file, "nominal,{},{},{},{:.2},{:.6},{:.6},{:.4},{:.4}",
             out_nominal.status,
             out_nominal.status == 0,
             out_nominal.iter,
             out_nominal.t4,
             out_nominal.tsfc_proxy,
             out_nominal.thrust_proxy,
             cfg.cycle.eta_comp,
             cfg.cycle.eta_turb)?;
    writeln!(file, "degraded_{},{},{},{},{:.2},{:.6},{:.6},{:.4},{:.4}",
             level,
             out_degraded.status,
             out_degraded.status == 0,
             out_degraded.iter,
             out_degraded.t4,
             out_degraded.tsfc_proxy,
             out_degraded.thrust_proxy,
             cfg.cycle.eta_comp * degradation.eta_comp_factor,
             cfg.cycle.eta_turb * degradation.eta_turb_factor)?;
    writeln!(file, "")?;
    writeln!(file, "# Delta metrics")?;
    writeln!(file, "# TSFC change: {:.2}%", tsfc_change_pct)?;
    writeln!(file, "# Thrust change: {:.2}%", thrust_change_pct)?;
    writeln!(file, "# T4 change: {:.1} K", t4_change_k)?;
    writeln!(file, "# Iteration change: {}", iter_change)?;

    eprintln!("[minotaur] comparison complete ({} degradation)", level);
    eprintln!("  Nominal:  status={} iter={} t4={:.1} tsfc={:.4} thrust={:.4}",
              out_nominal.status, out_nominal.iter, out_nominal.t4,
              out_nominal.tsfc_proxy, out_nominal.thrust_proxy);
    eprintln!("  Degraded: status={} iter={} t4={:.1} tsfc={:.4} thrust={:.4}",
              out_degraded.status, out_degraded.iter, out_degraded.t4,
              out_degraded.tsfc_proxy, out_degraded.thrust_proxy);
    eprintln!("  Delta:    TSFC={:+.2}% Thrust={:+.2}% T4={:+.1}K iter={:+}",
              tsfc_change_pct, thrust_change_pct, t4_change_k, iter_change);

    if json_output {
        let json_path = out_path.replace(".csv", ".json");
        let comparison = ComparisonOutput {
            manifest: create_manifest(cfg, cfg_text),
            nominal: ScenarioResult {
                scenario_name: "nominal".to_string(),
                status: out_nominal.status,
                status_name: ffi::status_name(out_nominal.status).to_string(),
                converged: out_nominal.status == 0,
                iterations: out_nominal.iter,
                t4: out_nominal.t4,
                tsfc_proxy: out_nominal.tsfc_proxy,
                thrust_proxy: out_nominal.thrust_proxy,
                eta_comp_effective: cfg.cycle.eta_comp,
                eta_turb_effective: cfg.cycle.eta_turb,
            },
            degraded: ScenarioResult {
                scenario_name: format!("degraded_{}", level),
                status: out_degraded.status,
                status_name: ffi::status_name(out_degraded.status).to_string(),
                converged: out_degraded.status == 0,
                iterations: out_degraded.iter,
                t4: out_degraded.t4,
                tsfc_proxy: out_degraded.tsfc_proxy,
                thrust_proxy: out_degraded.thrust_proxy,
                eta_comp_effective: cfg.cycle.eta_comp * degradation.eta_comp_factor,
                eta_turb_effective: cfg.cycle.eta_turb * degradation.eta_turb_factor,
            },
            delta: DeltaMetrics {
                tsfc_change_pct,
                thrust_change_pct,
                t4_change_k,
                iter_change,
            },
        };
        let json = serde_json::to_string_pretty(&comparison)?;
        fs::write(&json_path, json)?;
        eprintln!("[minotaur] JSON comparison: {}", json_path);
    }

    Ok(())
}

fn validate_config(cfg_path: &str) -> Result<()> {
    let cfg_text = fs::read_to_string(cfg_path)
        .with_context(|| format!("failed to read config: {}", cfg_path))?;

    let cfg: config::Root = toml::from_str(&cfg_text)
        .with_context(|| format!("failed to parse config: {}", cfg_path))?;

    cfg.validate()?;

    eprintln!("[minotaur] config valid: {}", cfg_path);
    eprintln!("  program: {} v{}", cfg.CSTNSystems.program, cfg.CSTNSystems.version);
    eprintln!("  solver: max_iter={}, tol={:.0e}, damping={}",
              cfg.solver.max_iter, cfg.solver.tol, cfg.solver.damping);
    eprintln!("  cycle: mach={}, alt_km={}, bpr={:?}, opr={:?}",
              cfg.cycle.mach, cfg.cycle.alt_km, cfg.cycle.bpr, cfg.cycle.opr);

    if let Some(sweep) = &cfg.sweep {
        eprintln!("  sweep: bpr=[{},{}]×{}, opr=[{},{}]×{}",
                  sweep.bpr_min, sweep.bpr_max, sweep.bpr_n,
                  sweep.opr_min, sweep.opr_max, sweep.opr_n);
    }

    if let Some(comp) = &cfg.components {
        eprintln!("  components: compressor={}, turbine={}, nozzle={}",
                  comp.compressor, comp.turbine, comp.nozzle);
    }

    if let Some(losses) = &cfg.losses {
        eprintln!("  losses: inlet={}, burner={}, turbine={}, nozzle={}",
                  losses.inlet, losses.burner, losses.turbine, losses.nozzle);
    }

    if let Some(deg) = &cfg.degradation {
        eprintln!("  degradation: eta_comp_factor={}, eta_turb_factor={}, loss_adder={}",
                  deg.eta_comp_factor, deg.eta_turb_factor, deg.loss_adder);
    }

    Ok(())
}

// v2.9: Run multi-objective optimization via NSGA-II
fn run_optimize(
    cfg: &config::Root,
    cfg_text: &str,
    out_path: &str,
    pop_size: usize,
    generations: usize,
    seed: u64,
    json_output: bool,
) -> Result<()> {
    use nsga2::{NSGA2, NSGA2Config, hypervolume_2d};

    // Configure optimizer
    let nsga_config = NSGA2Config {
        pop_size,
        generations,
        crossover_prob: 0.9,
        mutation_prob: 0.1,
        eta_c: 20.0,
        eta_m: 20.0,
        bounds: vec![
            (0.2, 1.5),                          // bpr
            (4.0, 16.0),                         // opr
            (0.75, 0.90),                        // eta_comp
            (0.80, 0.92),                        // eta_turb
        ],
        seed,
    };

    let mach = cfg.cycle.mach;
    let alt_km = cfg.cycle.alt_km;
    let t4_max = cfg.constraints.t4_max;

    eprintln!("[minotaur] starting NSGA-II optimization");
    eprintln!("  Population: {}, Generations: {}, Seed: {}", pop_size, generations, seed);

    let start = Instant::now();

    let mut optimizer = NSGA2::new(nsga_config.clone());

    // Evaluation function: returns (objectives, constraint_violation, status, outputs)
    let eval_fn = |x: &[f64]| -> (Vec<f64>, f64, i32, Vec<f64>) {
        let inp = ffi::MinotaurInput {
            mach,
            alt_km,
            bpr: x[0],
            opr: x[1],
            eta_comp: x[2],
            eta_turb: x[3],
            eta_nozz: cfg.cycle.eta_nozz,
            fuel_k: cfg.cycle.fuel_k,
            max_iter: cfg.solver.max_iter,
            tol: cfg.solver.tol,
            damping: cfg.solver.damping,
            mass_tol: cfg.invariants.mass_tol,
            energy_tol: cfg.invariants.energy_tol,
            t4_max,
        };

        let out = ffi::solve(inp);

        if out.status != 0 {
            // Penalize non-converged solutions
            (vec![1e6, 1e6], 1.0, out.status, vec![out.t4, out.iter as f64])
        } else {
            // Objectives: minimize TSFC, minimize -thrust (i.e., maximize thrust)
            let cv = if out.t4 > t4_max { out.t4 - t4_max } else { 0.0 };
            (vec![out.tsfc_proxy, -out.thrust_proxy], cv, out.status, vec![out.t4, out.iter as f64])
        }
    };

    let front = optimizer.optimize(eval_fn);
    let wall_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    eprintln!("[minotaur] optimization complete");
    eprintln!("  Pareto front size: {}", front.solutions.len());
    eprintln!("  Wall time: {:.1} ms", wall_time_ms);

    // Compute hypervolume
    let ref_point = (2.0, 0.0); // Reference point for hypervolume (max TSFC, min -thrust)
    let hv = hypervolume_2d(&front.solutions, ref_point);
    eprintln!("  Hypervolume (2D): {:.4}", hv);

    // Write CSV
    let mut file = fs::File::create(out_path)?;
    writeln!(file, "rank,crowding,bpr,opr,eta_comp,eta_turb,tsfc,thrust,t4,status")?;
    for sol in &front.solutions {
        writeln!(file, "{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.6},{:.6},{:.2},{}",
                 sol.rank,
                 if sol.crowding_distance.is_infinite() { -1.0 } else { sol.crowding_distance },
                 sol.x[0], sol.x[1], sol.x[2], sol.x[3],
                 sol.f[0], -sol.f[1],  // Convert back to positive thrust
                 sol.outputs[0],
                 sol.status)?;
    }

    eprintln!("[minotaur] Pareto front written to: {}", out_path);

    // Print top solutions
    eprintln!();
    eprintln!("  Top Pareto solutions:");
    eprintln!("  {:>6} {:>6} {:>8} {:>8} {:>10} {:>10} {:>8}",
              "BPR", "OPR", "eta_c", "eta_t", "TSFC", "Thrust", "T4");
    eprintln!("  {}", "-".repeat(70));

    let mut sorted = front.solutions.clone();
    sorted.sort_by(|a, b| a.f[0].partial_cmp(&b.f[0]).unwrap_or(std::cmp::Ordering::Equal));

    for (i, sol) in sorted.iter().take(10).enumerate() {
        eprintln!("  {:>6.3} {:>6.2} {:>8.4} {:>8.4} {:>10.6} {:>10.6} {:>8.1}",
                  sol.x[0], sol.x[1], sol.x[2], sol.x[3],
                  sol.f[0], -sol.f[1], sol.outputs[0]);
    }

    if json_output {
        let json_path = out_path.replace(".csv", ".json");

        let pareto_solutions: Vec<ParetoSolution> = front.solutions.iter().map(|sol| {
            ParetoSolution {
                bpr: sol.x[0],
                opr: sol.x[1],
                eta_comp: sol.x[2],
                eta_turb: sol.x[3],
                tsfc: sol.f[0],
                thrust: -sol.f[1],
                t4: sol.outputs[0],
                status: sol.status,
                rank: sol.rank,
                crowding_distance: sol.crowding_distance,
            }
        }).collect();

        let opt_output = OptimizationOutput {
            manifest: create_manifest(cfg, cfg_text),
            config: OptConfig {
                pop_size: nsga_config.pop_size,
                generations: nsga_config.generations,
                crossover_prob: nsga_config.crossover_prob,
                mutation_prob: nsga_config.mutation_prob,
                seed: nsga_config.seed,
                bounds: nsga_config.bounds,
                objectives: vec!["minimize TSFC".to_string(), "maximize Thrust".to_string()],
            },
            pareto_front: pareto_solutions,
            hypervolume: Some(hv),
            generations: front.generation,
            wall_time_ms,
        };

        let json = serde_json::to_string_pretty(&opt_output)?;
        fs::write(&json_path, json)?;
        eprintln!("[minotaur] JSON optimization results: {}", json_path);
    }

    Ok(())
}

// v2.8: Compute exact Jacobian via forward-mode AD
fn run_jacobian(cfg: &config::Root, cfg_text: &str, out_path: &str, json_output: bool) -> Result<()> {
    let bpr = cfg.cycle.bpr.context("cycle.bpr required")?;
    let opr = cfg.cycle.opr.context("cycle.opr required")?;

    let start = Instant::now();
    let result = ffi::compute_jacobian(
        cfg.cycle.mach,
        cfg.cycle.alt_km,
        bpr,
        opr,
        cfg.cycle.eta_comp,
        cfg.cycle.eta_turb,
        cfg.constraints.t4_max,
    );
    let wall_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    // Write CSV
    let mut file = fs::File::create(out_path)?;
    writeln!(file, "parameter,d_tsfc,d_thrust,d_t4")?;
    for (i, param) in result.param_names.iter().enumerate() {
        writeln!(file, "{},{:.8e},{:.8e},{:.8e}",
                 param, result.jacobian[i][0], result.jacobian[i][1], result.jacobian[i][2])?;
    }

    eprintln!("[minotaur] exact Jacobian computed via forward-mode AD");
    eprintln!("  Base values: TSFC={:.4}, Thrust={:.4}, T4={:.1} K",
              result.base_tsfc, result.base_thrust, result.base_t4);
    eprintln!("  Status: {} ({})", result.status, ffi::status_name(result.status));
    eprintln!("  Wall time: {:.2} ms", wall_time_ms);
    eprintln!();
    eprintln!("  Jacobian (dOutput/dParam):");
    eprintln!("  {:12} {:>14} {:>14} {:>14}", "Parameter", "dTSFC", "dThrust", "dT4");
    eprintln!("  {}", "-".repeat(56));
    for (i, param) in result.param_names.iter().enumerate() {
        eprintln!("  {:12} {:>14.6e} {:>14.6e} {:>14.4f}",
                  param, result.jacobian[i][0], result.jacobian[i][1], result.jacobian[i][2]);
    }

    if json_output {
        let json_path = out_path.replace(".csv", ".json");
        let mut base_values = HashMap::new();
        base_values.insert("tsfc".to_string(), result.base_tsfc);
        base_values.insert("thrust".to_string(), result.base_thrust);
        base_values.insert("t4".to_string(), result.base_t4);
        base_values.insert("mach".to_string(), cfg.cycle.mach);
        base_values.insert("alt_km".to_string(), cfg.cycle.alt_km);
        base_values.insert("bpr".to_string(), bpr);
        base_values.insert("opr".to_string(), opr);
        base_values.insert("eta_comp".to_string(), cfg.cycle.eta_comp);
        base_values.insert("eta_turb".to_string(), cfg.cycle.eta_turb);

        let jac_output = JacobianOutput {
            manifest: create_manifest(cfg, cfg_text),
            method: "forward-mode AD (dual numbers)".to_string(),
            parameters: result.param_names.iter().map(|s| s.to_string()).collect(),
            outputs: result.output_names.iter().map(|s| s.to_string()).collect(),
            jacobian: result.jacobian.iter().map(|row| row.to_vec()).collect(),
            base_values,
            status: result.status,
        };
        let json = serde_json::to_string_pretty(&jac_output)?;
        fs::write(&json_path, json)?;
        eprintln!("[minotaur] JSON jacobian: {}", json_path);
    }

    Ok(())
}

// Print detailed version information
fn print_version() {
    eprintln!("MINOTAUR - CSTNSystems Deterministic Reduced-Order Turbofan Cycle Solver");
    eprintln!();
    eprintln!("  CSTNSystems Program ID:  {}", CSTNSystems_PROGRAM_ID);
    eprintln!("  Solver Version:    {}", VERSION);
    eprintln!("  Schema Version:    {}", SCHEMA_VERSION);
    eprintln!("  Platform:          {}", std::env::consts::OS);
    eprintln!("  Architecture:      {}", std::env::consts::ARCH);
    eprintln!();
    eprintln!("Component Models (v2.4):");
    eprintln!("  - standard: Basic isentropic efficiency models");
    eprintln!("  - advanced: Includes polytropic/cooling/divergence effects");
    eprintln!();
    eprintln!("Degradation Levels (v2.4):");
    eprintln!("  - light:    5% efficiency loss, +1% pressure loss");
    eprintln!("  - moderate: 10% efficiency loss, +2% pressure loss");
    eprintln!("  - severe:   15% efficiency loss, +3% pressure loss");
    eprintln!();
    eprintln!("Automatic Differentiation (v2.8):");
    eprintln!("  - Forward-mode AD via dual numbers");
    eprintln!("  - Exact gradients without truncation error");
    eprintln!("  - Full Jacobian: 6 params x 3 outputs");
    eprintln!();
    eprintln!("Multi-Objective Optimization (v2.9):");
    eprintln!("  - NSGA-II algorithm");
    eprintln!("  - Bi-objective: minimize TSFC, maximize Thrust");
    eprintln!("  - Pareto front extraction with hypervolume");
    eprintln!();
    eprintln!("Production Release (v2.0):");
    eprintln!("  - Comprehensive API documentation");
    eprintln!("  - User guide with workflows and troubleshooting");
    eprintln!("  - Extensive test coverage with property tests");
    eprintln!("  - Performance benchmarks");
    eprintln!();
    eprintln!("Extended Physics (v2.1):");
    eprintln!("  - Variable specific heats Cp(T) via NASA polynomials");
    eprintln!("  - Real gas effects via Peng-Robinson EOS");
    eprintln!("  - Equilibrium and kinetic combustion models");
    eprintln!("  - Dissociation effects at high temperatures");
    eprintln!();
    eprintln!("CSTNSystems - Compact Subsonic Turbofan Numerical Systems");
}

// ============================================================================
// Main
// ============================================================================

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Version) => {
            print_version();
            return Ok(());
        }
        Some(Commands::Jacobian { json }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/jacobian.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);
            return run_jacobian(&cfg, &cfg_text, &out_path, json);
        }
        Some(Commands::Optimize { pop_size, generations, seed, json }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/pareto_front.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);
            return run_optimize(&cfg, &cfg_text, &out_path, pop_size, generations, seed, json);
        }
        Some(Commands::Validate) => {
            let cfg_path = args.config.context("--config required for validate")?;
            return validate_config(&cfg_path);
        }
        Some(Commands::Sensitivity { step }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/sensitivities.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            return run_sensitivity(&cfg, &out_path, step);
        }
        Some(Commands::Compare { level, json }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/comparison.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);
            return run_compare(&cfg, &cfg_text, &out_path, &level, json);
        }
        Some(Commands::Run { json }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/out_baseline.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);
            return run_single(&cfg, &cfg_text, &out_path, json);
        }
        Some(Commands::Sweep { json }) => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.unwrap_or_else(|| "results/out_sweep.csv".to_string());

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);
            return run_sweep(&cfg, &cfg_text, &out_path, json);
        }
        None => {
            let cfg_path = args.config.context("--config required")?;
            let out_path = args.out.context("--out required")?;

            let cfg_text = fs::read_to_string(&cfg_path)?;
            let cfg: config::Root = toml::from_str(&cfg_text)?;
            cfg.validate()?;

            eprintln!("[minotaur] {} v{} - {}", cfg.CSTNSystems.program, cfg.CSTNSystems.version, cfg.CSTNSystems.module);

            match args.mode.as_str() {
                "single" => run_single(&cfg, &cfg_text, &out_path, false),
                "sweep" => run_sweep(&cfg, &cfg_text, &out_path, false),
                _ => anyhow::bail!("unknown mode: {} (use 'single' or 'sweep')", args.mode),
            }
        }
    }
}
