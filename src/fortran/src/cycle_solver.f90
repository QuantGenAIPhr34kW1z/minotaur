module cycle_solver
  use, intrinsic :: iso_c_binding
  use minotaur_types
  use thermo
  use invariants
  implicit none

  ! Armijo line search parameters
  real(c_double), parameter :: ARMIJO_C = 1.0e-4_c_double
  real(c_double), parameter :: ARMIJO_RHO = 0.5_c_double
  integer, parameter :: MAX_LINE_SEARCH = 10

contains

  ! Compute residual for given state
  pure subroutine compute_residual(x_bpr, x_opr, inp, rf, target, resid, t4)
    real(c_double), intent(in) :: x_bpr, x_opr
    type(MinotaurInput), intent(in) :: inp
    real(c_double), intent(in) :: rf, target
    real(c_double), intent(out) :: resid, t4

    ! Reduced-order "thermal" proxy (monotone with OPR, penalized by inefficiencies)
    t4 = 900.0_c_double + 55.0_c_double*x_opr*(1.0_c_double/max(inp%eta_comp, 1.0e-6_c_double)) * rf

    ! Residual: couple BPR toward a smooth target; treat as nonlinear with damping
    resid = (x_bpr - target) + 0.03_c_double*(x_bpr*x_bpr - 0.36_c_double)
  end subroutine

  ! Check if state is admissible
  pure logical function is_admissible(x_bpr, t4, inp)
    real(c_double), intent(in) :: x_bpr, t4
    type(MinotaurInput), intent(in) :: inp

    is_admissible = .true.

    ! BPR bounds
    if (x_bpr < 0.0_c_double .or. x_bpr > 2.0_c_double) then
      is_admissible = .false.
      return
    end if

    ! Thermal constraint
    if (t4 > inp%t4_max) then
      is_admissible = .false.
      return
    end if

    ! Finiteness
    if (.not. is_finite(x_bpr) .or. .not. is_finite(t4)) then
      is_admissible = .false.
      return
    end if
  end function

  ! Armijo backtracking line search
  subroutine line_search(x_bpr, x_opr, inp, rf, target, direction, &
                         new_bpr, new_resid, new_t4, alpha, ls_steps)
    real(c_double), intent(in) :: x_bpr, x_opr
    type(MinotaurInput), intent(in) :: inp
    real(c_double), intent(in) :: rf, target, direction
    real(c_double), intent(out) :: new_bpr, new_resid, new_t4, alpha
    integer, intent(out) :: ls_steps

    real(c_double) :: current_resid, current_t4, trial_bpr, trial_resid, trial_t4
    real(c_double) :: f0, slope
    integer :: i

    ! Compute current residual
    call compute_residual(x_bpr, x_opr, inp, rf, target, current_resid, current_t4)
    f0 = abs(current_resid)
    slope = -abs(current_resid)  ! Descent direction

    alpha = 1.0_c_double
    ls_steps = 0

    do i = 1, MAX_LINE_SEARCH
      trial_bpr = x_bpr + alpha * direction

      ! Project to admissible region
      trial_bpr = max(0.0_c_double, min(2.0_c_double, trial_bpr))

      call compute_residual(trial_bpr, x_opr, inp, rf, target, trial_resid, trial_t4)

      ! Armijo condition: f(x + alpha*d) <= f(x) + c*alpha*slope
      if (abs(trial_resid) <= f0 + ARMIJO_C * alpha * slope) then
        new_bpr = trial_bpr
        new_resid = trial_resid
        new_t4 = trial_t4
        return
      end if

      alpha = alpha * ARMIJO_RHO
      ls_steps = ls_steps + 1
    end do

    ! Line search failed, take damped step anyway
    new_bpr = max(0.0_c_double, min(2.0_c_double, x_bpr + alpha * direction))
    call compute_residual(new_bpr, x_opr, inp, rf, target, new_resid, new_t4)
  end subroutine

  ! Main solver with convergence history
  subroutine solve_cycle_with_history(inp, out, diag)
    type(MinotaurInput), intent(in) :: inp
    type(MinotaurOutput), intent(inout) :: out
    type(MinotaurDiagnostics), intent(inout) :: diag

    integer :: k, ls_steps
    real(c_double) :: x_bpr, x_opr
    real(c_double) :: rf, t4, resid, target
    real(c_double) :: direction, alpha, new_bpr, new_resid, new_t4
    real(c_double) :: mass_r, energy_r
    real(c_double) :: best_resid
    logical :: admissible

    x_bpr = inp%bpr
    x_opr = inp%opr

    out%status = STATUS_MAXITER
    out%iter = 0
    diag%history_len = 0
    diag%line_search_steps = 0
    diag%last_admissible_bpr = x_bpr
    diag%last_admissible_t4 = 0.0_c_double

    rf = regime_factor(inp%mach, inp%alt_km)

    ! Synthetic target function
    target = 0.6_c_double + 0.02_c_double*(inp%opr - 8.0_c_double)

    ! Compute initial residual
    call compute_residual(x_bpr, x_opr, inp, rf, target, resid, t4)
    diag%initial_residual = abs(resid)
    best_resid = abs(resid)
    diag%best_residual = best_resid

    do k = 1, inp%max_iter
      ! Record convergence history
      if (diag%history_len < MAX_HISTORY) then
        diag%history_len = diag%history_len + 1
        diag%history(diag%history_len)%iteration = k - 1
        diag%history(diag%history_len)%residual_norm = abs(resid)
        diag%history(diag%history_len)%bpr = x_bpr
        diag%history(diag%history_len)%t4 = t4
        diag%history(diag%history_len)%step_size = 0.0_c_double
        admissible = is_admissible(x_bpr, t4, inp)
        if (admissible) then
          diag%history(diag%history_len)%admissible = 1
          diag%last_admissible_bpr = x_bpr
          diag%last_admissible_t4 = t4
        else
          diag%history(diag%history_len)%admissible = 0
        end if
      end if

      ! Check convergence
      if (abs(resid) < inp%tol) then
        out%status = STATUS_OK
        out%iter = k
        out%t4 = t4
        out%final_bpr = x_bpr
        out%final_residual = abs(resid)
        exit
      end if

      ! Check for non-physical state
      if (.not. is_finite(resid) .or. .not. is_finite(t4)) then
        out%status = STATUS_NONPHYSICAL
        out%iter = k
        out%t4 = t4
        out%final_bpr = x_bpr
        out%final_residual = abs(resid)
        exit
      end if

      ! Check constraint violation
      if (t4 > inp%t4_max) then
        out%status = STATUS_CONSTRAINT_VIOL
        out%iter = k
        out%t4 = t4
        out%final_bpr = x_bpr
        out%final_residual = abs(resid)
        exit
      end if

      ! Check for divergence (residual growing)
      if (k > 10 .and. abs(resid) > 10.0_c_double * diag%initial_residual) then
        out%status = STATUS_DIVERGED
        out%iter = k
        out%t4 = t4
        out%final_bpr = x_bpr
        out%final_residual = abs(resid)
        exit
      end if

      ! Compute Newton direction
      direction = -inp%damping * resid

      ! Line search
      call line_search(x_bpr, x_opr, inp, rf, target, direction, &
                       new_bpr, new_resid, new_t4, alpha, ls_steps)

      diag%line_search_steps = diag%line_search_steps + ls_steps

      ! Update step size in history
      if (diag%history_len <= MAX_HISTORY) then
        diag%history(diag%history_len)%step_size = alpha * abs(direction)
      end if

      ! Update state
      x_bpr = new_bpr
      resid = new_resid
      t4 = new_t4

      ! Track best residual
      if (abs(resid) < best_resid) then
        best_resid = abs(resid)
        diag%best_residual = best_resid
      end if
    end do

    ! If we exited loop without converging, record final state
    if (out%status == STATUS_MAXITER) then
      out%iter = inp%max_iter
      out%t4 = t4
      out%final_bpr = x_bpr
      out%final_residual = abs(resid)
    end if

    ! Invariants computed post-solve (hard gates)
    mass_r = mass_residual(x_bpr, x_opr)
    energy_r = energy_residual(t4, inp%t4_max)

    out%mass_resid = mass_r
    out%energy_resid = energy_r

    if (out%status == STATUS_OK) then
      if (mass_r > inp%mass_tol .or. energy_r > inp%energy_tol) then
        out%status = STATUS_INVARIANT_VIOL
      end if
    end if

    ! Proxies (synthetic performance metrics)
    out%tsfc_proxy = (1.2_c_double - 0.18_c_double*x_bpr) * &
                     (1.0_c_double + 0.02_c_double*(x_opr-8.0_c_double))
    out%thrust_proxy = (0.8_c_double + 0.12_c_double*x_opr) * &
                       (1.0_c_double - 0.10_c_double*x_bpr) * rf
  end subroutine

  ! Simple solver (backward compatible)
  subroutine solve_cycle(inp, out)
    type(MinotaurInput), intent(in) :: inp
    type(MinotaurOutput), intent(inout) :: out
    type(MinotaurDiagnostics) :: diag

    call solve_cycle_with_history(inp, out, diag)
  end subroutine

  ! Extended solver with component models and degradation (v2.4)
  subroutine solve_cycle_extended(inp_ext, out, diag)
    type(MinotaurInputExt), intent(in) :: inp_ext
    type(MinotaurOutput), intent(inout) :: out
    type(MinotaurDiagnostics), intent(inout) :: diag

    type(MinotaurInput) :: inp
    real(c_double) :: eta_comp_eff, eta_turb_eff
    real(c_double) :: loss_factor

    ! Apply degradation factors
    eta_comp_eff = inp_ext%eta_comp * inp_ext%eta_comp_factor
    eta_turb_eff = inp_ext%eta_turb * inp_ext%eta_turb_factor

    ! Clamp efficiencies to valid range
    eta_comp_eff = max(0.5_c_double, min(1.0_c_double, eta_comp_eff))
    eta_turb_eff = max(0.5_c_double, min(1.0_c_double, eta_turb_eff))

    ! Calculate loss factor from component losses
    loss_factor = 1.0_c_double - (inp_ext%inlet_loss + inp_ext%burner_loss + &
                                  inp_ext%turbine_mech_loss + inp_ext%nozzle_loss + &
                                  inp_ext%loss_adder)
    loss_factor = max(0.5_c_double, min(1.0_c_double, loss_factor))

    ! Copy to base input with effective values
    inp%mach = inp_ext%mach
    inp%alt_km = inp_ext%alt_km
    inp%bpr = inp_ext%bpr
    inp%opr = inp_ext%opr
    inp%eta_comp = eta_comp_eff
    inp%eta_turb = eta_turb_eff
    inp%eta_nozz = inp_ext%eta_nozz * loss_factor
    inp%fuel_k = inp_ext%fuel_k
    inp%max_iter = inp_ext%max_iter
    inp%tol = inp_ext%tol
    inp%damping = inp_ext%damping
    inp%mass_tol = inp_ext%mass_tol
    inp%energy_tol = inp_ext%energy_tol
    inp%t4_max = inp_ext%t4_max

    ! Run solver with effective parameters
    call solve_cycle_with_history(inp, out, diag)

    ! Adjust outputs based on component models
    if (inp_ext%compressor_model == MODEL_ADVANCED) then
      ! Advanced compressor model: slightly different thermal profile
      out%t4 = out%t4 * (1.0_c_double + 0.02_c_double * (1.0_c_double - eta_comp_eff))
    end if

    if (inp_ext%turbine_model == MODEL_ADVANCED) then
      ! Advanced turbine model: accounts for cooling losses
      out%thrust_proxy = out%thrust_proxy * 0.98_c_double
    end if

    if (inp_ext%nozzle_model == MODEL_ADVANCED) then
      ! Advanced nozzle model: divergence losses
      out%thrust_proxy = out%thrust_proxy * 0.985_c_double
    end if

    ! Apply degradation penalty to TSFC
    if (inp_ext%is_degraded == 1) then
      out%tsfc_proxy = out%tsfc_proxy / (inp_ext%eta_comp_factor * inp_ext%eta_turb_factor)
    end if
  end subroutine

  ! Convert base input to extended input with defaults
  pure subroutine input_to_extended(inp, inp_ext)
    type(MinotaurInput), intent(in) :: inp
    type(MinotaurInputExt), intent(out) :: inp_ext

    inp_ext%mach = inp%mach
    inp_ext%alt_km = inp%alt_km
    inp_ext%bpr = inp%bpr
    inp_ext%opr = inp%opr
    inp_ext%eta_comp = inp%eta_comp
    inp_ext%eta_turb = inp%eta_turb
    inp_ext%eta_nozz = inp%eta_nozz
    inp_ext%fuel_k = inp%fuel_k
    inp_ext%max_iter = inp%max_iter
    inp_ext%tol = inp%tol
    inp_ext%damping = inp%damping
    inp_ext%mass_tol = inp%mass_tol
    inp_ext%energy_tol = inp%energy_tol
    inp_ext%t4_max = inp%t4_max

    ! Default component models (standard)
    inp_ext%compressor_model = MODEL_STANDARD
    inp_ext%turbine_model = MODEL_STANDARD
    inp_ext%nozzle_model = MODEL_STANDARD

    ! Default loss coefficients
    inp_ext%inlet_loss = 0.02_c_double
    inp_ext%burner_loss = 0.04_c_double
    inp_ext%turbine_mech_loss = 0.02_c_double
    inp_ext%nozzle_loss = 0.01_c_double

    ! No degradation by default
    inp_ext%eta_comp_factor = 1.0_c_double
    inp_ext%eta_turb_factor = 1.0_c_double
    inp_ext%loss_adder = 0.0_c_double
    inp_ext%is_degraded = 0
  end subroutine

end module
