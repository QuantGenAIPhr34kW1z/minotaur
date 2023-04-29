"""
MINOTAUR: Deterministic Reduced-Order Turbofan Cycle Solver

A Python interface to the CSTNSystems/MINOTAUR turbofan cycle solver.

Example:
    >>> import minotaur
    >>> result = minotaur.solve(mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0)
    >>> print(f"T4 = {result.t4:.1f} K, TSFC = {result.tsfc_proxy:.4f}")
"""

from typing import Dict, List, Optional, Tuple, Union
import numpy as np

try:
    from ._minotaur import (
        SolverResult,
        solve as _solve,
        solve_extended as _solve_extended,
        sweep as _sweep,
        sensitivity as _sensitivity,
        compare_degradation as _compare_degradation,
        version as _version,
        STATUS_OK,
        STATUS_MAXITER,
        STATUS_DIVERGED,
        STATUS_INVARIANT_VIOL,
        STATUS_CONSTRAINT_VIOL,
        STATUS_NONPHYSICAL,
        SCHEMA_VERSION,
    )
except ImportError:
    # Fallback for development/testing without compiled extension
    pass

__version__ = "2.9.0"
__all__ = [
    "solve",
    "solve_extended",
    "sweep",
    "sensitivity",
    "compare_degradation",
    "version",
    "SolverResult",
    "STATUS_OK",
    "STATUS_MAXITER",
    "STATUS_DIVERGED",
    "STATUS_INVARIANT_VIOL",
    "STATUS_CONSTRAINT_VIOL",
    "STATUS_NONPHYSICAL",
]


def solve(
    mach: float,
    alt_km: float,
    bpr: float,
    opr: float,
    eta_comp: float = 0.82,
    eta_turb: float = 0.86,
    eta_nozz: float = 0.95,
    fuel_k: float = 1.0,
    max_iter: int = 200,
    tol: float = 1e-10,
    damping: float = 0.5,
    mass_tol: float = 1e-9,
    energy_tol: float = 1e-9,
    t4_max: float = 1400.0,
) -> "SolverResult":
    """
    Solve a single turbofan cycle operating point.

    Parameters
    ----------
    mach : float
        Flight Mach number [0, 0.95]
    alt_km : float
        Altitude in kilometers [0, 20]
    bpr : float
        Bypass ratio
    opr : float
        Overall pressure ratio
    eta_comp : float, optional
        Compressor isentropic efficiency (default: 0.82)
    eta_turb : float, optional
        Turbine isentropic efficiency (default: 0.86)
    eta_nozz : float, optional
        Nozzle efficiency (default: 0.95)
    fuel_k : float, optional
        Fuel parameter (default: 1.0)
    max_iter : int, optional
        Maximum solver iterations (default: 200)
    tol : float, optional
        Convergence tolerance (default: 1e-10)
    damping : float, optional
        Newton damping factor (default: 0.5)
    mass_tol : float, optional
        Mass conservation tolerance (default: 1e-9)
    energy_tol : float, optional
        Energy conservation tolerance (default: 1e-9)
    t4_max : float, optional
        Maximum turbine inlet temperature [K] (default: 1400.0)

    Returns
    -------
    SolverResult
        Result object with status, convergence info, and performance metrics.

    Examples
    --------
    >>> result = minotaur.solve(mach=0.65, alt_km=8.0, bpr=0.6, opr=8.0)
    >>> print(result)
    SolverResult(status=0, converged=True, iter=12, t4=1285.3, tsfc=0.9123, thrust=1.0341)
    """
    return _solve(
        mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz,
        fuel_k, max_iter, tol, damping, mass_tol, energy_tol, t4_max
    )


def solve_extended(
    mach: float,
    alt_km: float,
    bpr: float,
    opr: float,
    eta_comp: float = 0.82,
    eta_turb: float = 0.86,
    eta_nozz: float = 0.95,
    fuel_k: float = 1.0,
    max_iter: int = 200,
    tol: float = 1e-10,
    damping: float = 0.5,
    mass_tol: float = 1e-9,
    energy_tol: float = 1e-9,
    t4_max: float = 1400.0,
    compressor_model: int = 0,
    turbine_model: int = 0,
    nozzle_model: int = 0,
    inlet_loss: float = 0.02,
    burner_loss: float = 0.04,
    turbine_mech_loss: float = 0.02,
    nozzle_loss: float = 0.01,
    eta_comp_factor: float = 1.0,
    eta_turb_factor: float = 1.0,
    loss_adder: float = 0.0,
) -> "SolverResult":
    """
    Solve with extended options (component models, losses, degradation).

    Parameters
    ----------
    mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz, fuel_k,
    max_iter, tol, damping, mass_tol, energy_tol, t4_max
        See solve() for descriptions.
    compressor_model : int, optional
        0 = standard (isentropic), 1 = advanced (polytropic)
    turbine_model : int, optional
        0 = standard, 1 = advanced (cooled)
    nozzle_model : int, optional
        0 = standard, 1 = advanced (divergence losses)
    inlet_loss : float, optional
        Inlet pressure loss coefficient (default: 0.02)
    burner_loss : float, optional
        Combustor pressure loss (default: 0.04)
    turbine_mech_loss : float, optional
        Turbine mechanical loss (default: 0.02)
    nozzle_loss : float, optional
        Nozzle velocity loss (default: 0.01)
    eta_comp_factor : float, optional
        Compressor efficiency multiplier for degradation (default: 1.0)
    eta_turb_factor : float, optional
        Turbine efficiency multiplier for degradation (default: 1.0)
    loss_adder : float, optional
        Additional pressure loss (default: 0.0)

    Returns
    -------
    SolverResult
        Result object with status, convergence info, and performance metrics.
    """
    return _solve_extended(
        mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz,
        fuel_k, max_iter, tol, damping, mass_tol, energy_tol, t4_max,
        compressor_model, turbine_model, nozzle_model,
        inlet_loss, burner_loss, turbine_mech_loss, nozzle_loss,
        eta_comp_factor, eta_turb_factor, loss_adder
    )


def sweep(
    bpr_values: np.ndarray,
    opr_values: np.ndarray,
    mach: float = 0.65,
    alt_km: float = 8.0,
    eta_comp: float = 0.82,
    eta_turb: float = 0.86,
    eta_nozz: float = 0.95,
    t4_max: float = 1400.0,
) -> Dict[str, np.ndarray]:
    """
    Run a parameter sweep over BPR and OPR ranges.

    Parameters
    ----------
    bpr_values : np.ndarray
        Array of bypass ratio values to sweep
    opr_values : np.ndarray
        Array of overall pressure ratio values to sweep
    mach : float, optional
        Flight Mach number (default: 0.65)
    alt_km : float, optional
        Altitude in kilometers (default: 8.0)
    eta_comp : float, optional
        Compressor efficiency (default: 0.82)
    eta_turb : float, optional
        Turbine efficiency (default: 0.86)
    eta_nozz : float, optional
        Nozzle efficiency (default: 0.95)
    t4_max : float, optional
        Maximum T4 [K] (default: 1400.0)

    Returns
    -------
    dict
        Dictionary with NumPy arrays for each output:
        - bpr, opr: Input parameter values
        - status, iterations: Solver status and iteration counts
        - t4, tsfc, thrust: Performance metrics
        - n_bpr, n_opr: Grid dimensions

    Examples
    --------
    >>> bpr = np.linspace(0.2, 1.2, 21)
    >>> opr = np.linspace(4, 14, 21)
    >>> results = minotaur.sweep(bpr, opr)
    >>> print(f"Converged: {np.sum(results['status'] == 0)}/{len(results['status'])}")
    """
    bpr_arr = np.asarray(bpr_values, dtype=np.float64)
    opr_arr = np.asarray(opr_values, dtype=np.float64)
    return _sweep(bpr_arr, opr_arr, mach, alt_km, eta_comp, eta_turb, eta_nozz, t4_max)


def sensitivity(
    mach: float = 0.65,
    alt_km: float = 8.0,
    bpr: float = 0.6,
    opr: float = 8.0,
    eta_comp: float = 0.82,
    eta_turb: float = 0.86,
    step: float = 1e-6,
) -> Dict[str, Union[np.ndarray, float, List[str]]]:
    """
    Compute local sensitivities via central finite differences.

    Parameters
    ----------
    mach : float, optional
        Nominal Mach number (default: 0.65)
    alt_km : float, optional
        Nominal altitude [km] (default: 8.0)
    bpr : float, optional
        Nominal bypass ratio (default: 0.6)
    opr : float, optional
        Nominal overall pressure ratio (default: 8.0)
    eta_comp : float, optional
        Nominal compressor efficiency (default: 0.82)
    eta_turb : float, optional
        Nominal turbine efficiency (default: 0.86)
    step : float, optional
        Relative step size for finite differences (default: 1e-6)

    Returns
    -------
    dict
        Dictionary with:
        - parameters: List of parameter names
        - outputs: List of output names (tsfc, thrust, t4)
        - jacobian: 6x3 NumPy array of sensitivities
        - base_tsfc, base_thrust, base_t4: Nominal output values

    Examples
    --------
    >>> sens = minotaur.sensitivity(bpr=0.6, opr=8.0)
    >>> print(f"dTSFC/dBPR = {sens['jacobian'][0, 0]:.3f}")
    """
    return _sensitivity(mach, alt_km, bpr, opr, eta_comp, eta_turb, step)


def compare_degradation(
    mach: float = 0.65,
    alt_km: float = 8.0,
    bpr: float = 0.6,
    opr: float = 8.0,
    eta_comp: float = 0.82,
    eta_turb: float = 0.86,
    degradation_level: str = "moderate",
) -> Tuple["SolverResult", "SolverResult", float, float, float]:
    """
    Compare nominal vs degraded performance.

    Parameters
    ----------
    mach : float, optional
        Flight Mach number (default: 0.65)
    alt_km : float, optional
        Altitude [km] (default: 8.0)
    bpr : float, optional
        Bypass ratio (default: 0.6)
    opr : float, optional
        Overall pressure ratio (default: 8.0)
    eta_comp : float, optional
        Compressor efficiency (default: 0.82)
    eta_turb : float, optional
        Turbine efficiency (default: 0.86)
    degradation_level : str, optional
        One of "light", "moderate", "severe" (default: "moderate")

    Returns
    -------
    tuple
        (nominal_result, degraded_result, tsfc_change_%, thrust_change_%, t4_change_K)

    Examples
    --------
    >>> nom, deg, dtsfc, dthrust, dt4 = minotaur.compare_degradation(level="moderate")
    >>> print(f"TSFC change: {dtsfc:+.1f}%, T4 change: {dt4:+.1f} K")
    """
    return _compare_degradation(mach, alt_km, bpr, opr, eta_comp, eta_turb, degradation_level)


def version() -> Tuple[str, str]:
    """
    Get version information.

    Returns
    -------
    tuple
        (solver_version, schema_version)
    """
    return _version()
