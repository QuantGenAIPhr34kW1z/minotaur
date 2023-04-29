use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Root {
    pub CSTNSystems: CSTNSystems,
    pub solver: Solver,
    pub invariants: Invariants,
    pub constraints: Constraints,
    pub cycle: Cycle,
    pub sweep: Option<Sweep>,
    pub components: Option<Components>,     // v2.4: Component model selection
    pub losses: Option<Losses>,             // v2.4: Loss coefficients
    pub degradation: Option<Degradation>,   // v2.4: Degradation scenarios
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CSTNSystems {
    pub program: String,
    pub module: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Solver {
    pub max_iter: i32,
    pub tol: f64,
    pub damping: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Invariants {
    pub mass_tol: f64,
    pub energy_tol: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Constraints {
    pub t4_max: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cycle {
    pub mach: f64,
    pub alt_km: f64,
    pub bpr: Option<f64>,
    pub opr: Option<f64>,
    pub eta_comp: f64,
    pub eta_turb: f64,
    pub eta_nozz: f64,
    pub fuel_k: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sweep {
    pub bpr_min: f64,
    pub bpr_max: f64,
    pub bpr_n: usize,
    pub opr_min: f64,
    pub opr_max: f64,
    pub opr_n: usize,
}

// v2.4: Component model configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Components {
    /// Compressor model: "standard" or "advanced"
    #[serde(default = "default_model")]
    pub compressor: String,
    /// Turbine model: "standard" or "advanced"
    #[serde(default = "default_model")]
    pub turbine: String,
    /// Nozzle model: "standard" or "advanced"
    #[serde(default = "default_model")]
    pub nozzle: String,
}

fn default_model() -> String {
    "standard".to_string()
}

impl Default for Components {
    fn default() -> Self {
        Self {
            compressor: "standard".to_string(),
            turbine: "standard".to_string(),
            nozzle: "standard".to_string(),
        }
    }
}

impl Components {
    pub fn compressor_id(&self) -> i32 {
        match self.compressor.as_str() {
            "advanced" => 1,
            _ => 0,
        }
    }

    pub fn turbine_id(&self) -> i32 {
        match self.turbine.as_str() {
            "advanced" => 1,
            _ => 0,
        }
    }

    pub fn nozzle_id(&self) -> i32 {
        match self.nozzle.as_str() {
            "advanced" => 1,
            _ => 0,
        }
    }
}

// v2.4: Loss coefficient configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Losses {
    /// Inlet pressure loss coefficient (default: 0.02)
    #[serde(default = "default_inlet_loss")]
    pub inlet: f64,
    /// Combustor pressure loss coefficient (default: 0.04)
    #[serde(default = "default_burner_loss")]
    pub burner: f64,
    /// Turbine mechanical loss coefficient (default: 0.02)
    #[serde(default = "default_turbine_loss")]
    pub turbine: f64,
    /// Nozzle velocity loss coefficient (default: 0.01)
    #[serde(default = "default_nozzle_loss")]
    pub nozzle: f64,
}

fn default_inlet_loss() -> f64 { 0.02 }
fn default_burner_loss() -> f64 { 0.04 }
fn default_turbine_loss() -> f64 { 0.02 }
fn default_nozzle_loss() -> f64 { 0.01 }

impl Default for Losses {
    fn default() -> Self {
        Self {
            inlet: 0.02,
            burner: 0.04,
            turbine: 0.02,
            nozzle: 0.01,
        }
    }
}

// v2.4: Degradation scenario configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Degradation {
    /// Compressor efficiency multiplier (1.0 = nominal, <1.0 = degraded)
    #[serde(default = "default_one")]
    pub eta_comp_factor: f64,
    /// Turbine efficiency multiplier (1.0 = nominal, <1.0 = degraded)
    #[serde(default = "default_one")]
    pub eta_turb_factor: f64,
    /// Additional pressure loss coefficient
    #[serde(default = "default_zero")]
    pub loss_adder: f64,
    /// Scenario name/description
    #[serde(default)]
    pub scenario_name: String,
}

fn default_one() -> f64 { 1.0 }
fn default_zero() -> f64 { 0.0 }

impl Default for Degradation {
    fn default() -> Self {
        Self {
            eta_comp_factor: 1.0,
            eta_turb_factor: 1.0,
            loss_adder: 0.0,
            scenario_name: "nominal".to_string(),
        }
    }
}

impl Degradation {
    /// Create light degradation scenario (5% efficiency loss)
    pub fn light() -> Self {
        Self {
            eta_comp_factor: 0.95,
            eta_turb_factor: 0.97,
            loss_adder: 0.01,
            scenario_name: "light".to_string(),
        }
    }

    /// Create moderate degradation scenario (10% efficiency loss)
    pub fn moderate() -> Self {
        Self {
            eta_comp_factor: 0.90,
            eta_turb_factor: 0.94,
            loss_adder: 0.02,
            scenario_name: "moderate".to_string(),
        }
    }

    /// Create severe degradation scenario (15% efficiency loss)
    pub fn severe() -> Self {
        Self {
            eta_comp_factor: 0.85,
            eta_turb_factor: 0.91,
            loss_adder: 0.03,
            scenario_name: "severe".to_string(),
        }
    }

    /// Check if this represents a degraded condition
    pub fn is_degraded(&self) -> bool {
        self.eta_comp_factor < 1.0 || self.eta_turb_factor < 1.0 || self.loss_adder > 0.0
    }
}

impl Root {
    pub fn validate(&self) -> Result<()> {
        if self.CSTNSystems.program != "CSTNSystems" {
            bail!("CSTNSystems.program must be CSTNSystems");
        }
        if self.solver.max_iter <= 0 || self.solver.max_iter > 10_000 {
            bail!("solver.max_iter must be in [1, 10000]");
        }
        if !(0.0 < self.solver.damping && self.solver.damping <= 1.0) {
            bail!("solver.damping must be in (0, 1]");
        }
        if self.solver.tol <= 0.0 {
            bail!("solver.tol must be positive");
        }
        if self.constraints.t4_max <= 0.0 {
            bail!("constraints.t4_max must be positive");
        }
        if !(0.0..=0.95).contains(&self.cycle.mach) {
            bail!("cycle.mach must be in [0, 0.95]");
        }
        if !(0.0..=20.0).contains(&self.cycle.alt_km) {
            bail!("cycle.alt_km must be in [0, 20]");
        }
        if !(0.0..=1.0).contains(&self.cycle.eta_comp) {
            bail!("cycle.eta_comp must be in [0, 1]");
        }
        if !(0.0..=1.0).contains(&self.cycle.eta_turb) {
            bail!("cycle.eta_turb must be in [0, 1]");
        }
        if !(0.0..=1.0).contains(&self.cycle.eta_nozz) {
            bail!("cycle.eta_nozz must be in [0, 1]");
        }

        // Validate sweep if present
        if let Some(ref sweep) = self.sweep {
            if sweep.bpr_n == 0 || sweep.opr_n == 0 {
                bail!("sweep.bpr_n and sweep.opr_n must be >= 1");
            }
            if sweep.bpr_min > sweep.bpr_max {
                bail!("sweep.bpr_min must be <= sweep.bpr_max");
            }
            if sweep.opr_min > sweep.opr_max {
                bail!("sweep.opr_min must be <= sweep.opr_max");
            }
        }

        // v2.4: Validate components if present
        if let Some(ref comp) = self.components {
            let valid_models = ["standard", "advanced"];
            if !valid_models.contains(&comp.compressor.as_str()) {
                bail!("components.compressor must be 'standard' or 'advanced'");
            }
            if !valid_models.contains(&comp.turbine.as_str()) {
                bail!("components.turbine must be 'standard' or 'advanced'");
            }
            if !valid_models.contains(&comp.nozzle.as_str()) {
                bail!("components.nozzle must be 'standard' or 'advanced'");
            }
        }

        // v2.4: Validate losses if present
        if let Some(ref losses) = self.losses {
            if !(0.0..=0.5).contains(&losses.inlet) {
                bail!("losses.inlet must be in [0, 0.5]");
            }
            if !(0.0..=0.5).contains(&losses.burner) {
                bail!("losses.burner must be in [0, 0.5]");
            }
            if !(0.0..=0.5).contains(&losses.turbine) {
                bail!("losses.turbine must be in [0, 0.5]");
            }
            if !(0.0..=0.5).contains(&losses.nozzle) {
                bail!("losses.nozzle must be in [0, 0.5]");
            }
        }

        // v2.4: Validate degradation if present
        if let Some(ref deg) = self.degradation {
            if !(0.5..=1.0).contains(&deg.eta_comp_factor) {
                bail!("degradation.eta_comp_factor must be in [0.5, 1.0]");
            }
            if !(0.5..=1.0).contains(&deg.eta_turb_factor) {
                bail!("degradation.eta_turb_factor must be in [0.5, 1.0]");
            }
            if !(0.0..=0.2).contains(&deg.loss_adder) {
                bail!("degradation.loss_adder must be in [0, 0.2]");
            }
        }

        Ok(())
    }
}
