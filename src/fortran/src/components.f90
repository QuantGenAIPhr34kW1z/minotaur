module components
  use, intrinsic :: iso_c_binding
  use minotaur_types
  implicit none

  ! Component model identifiers
  integer, parameter :: MODEL_STANDARD = 0
  integer, parameter :: MODEL_ADVANCED = 1

  ! Loss coefficient defaults
  real(c_double), parameter :: DEFAULT_INLET_LOSS = 0.02_c_double
  real(c_double), parameter :: DEFAULT_BURNER_LOSS = 0.04_c_double
  real(c_double), parameter :: DEFAULT_TURBINE_LOSS = 0.02_c_double
  real(c_double), parameter :: DEFAULT_NOZZLE_LOSS = 0.01_c_double

  ! Extended input with component models and loss coefficients
  type, bind(C) :: ComponentConfig
     integer(c_int) :: compressor_model    ! 0=standard, 1=advanced
     integer(c_int) :: turbine_model       ! 0=standard, 1=advanced
     integer(c_int) :: nozzle_model        ! 0=standard, 1=advanced
     real(c_double) :: inlet_loss          ! Pressure loss coefficient
     real(c_double) :: burner_loss         ! Combustor pressure loss
     real(c_double) :: turbine_loss        ! Turbine mechanical loss
     real(c_double) :: nozzle_loss         ! Nozzle velocity loss
  end type

  ! Degradation scenario parameters
  type, bind(C) :: DegradationParams
     real(c_double) :: eta_comp_factor     ! Compressor efficiency multiplier (1.0 = nominal)
     real(c_double) :: eta_turb_factor     ! Turbine efficiency multiplier
     real(c_double) :: inlet_loss_adder    ! Additional inlet pressure loss
     real(c_double) :: burner_loss_adder   ! Additional burner pressure loss
     integer(c_int) :: is_degraded         ! 1 if degradation applied, 0 otherwise
  end type

contains

  ! Initialize default component configuration
  pure subroutine init_component_config(cc)
    type(ComponentConfig), intent(out) :: cc
    cc%compressor_model = MODEL_STANDARD
    cc%turbine_model = MODEL_STANDARD
    cc%nozzle_model = MODEL_STANDARD
    cc%inlet_loss = DEFAULT_INLET_LOSS
    cc%burner_loss = DEFAULT_BURNER_LOSS
    cc%turbine_loss = DEFAULT_TURBINE_LOSS
    cc%nozzle_loss = DEFAULT_NOZZLE_LOSS
  end subroutine

  ! Initialize nominal (no degradation) parameters
  pure subroutine init_nominal_degradation(dp)
    type(DegradationParams), intent(out) :: dp
    dp%eta_comp_factor = 1.0_c_double
    dp%eta_turb_factor = 1.0_c_double
    dp%inlet_loss_adder = 0.0_c_double
    dp%burner_loss_adder = 0.0_c_double
    dp%is_degraded = 0
  end subroutine

  ! Standard compressor model: simple isentropic efficiency
  pure function compressor_standard(pi_c, eta_c, t_in) result(t_out)
    real(c_double), intent(in) :: pi_c      ! Pressure ratio
    real(c_double), intent(in) :: eta_c     ! Isentropic efficiency
    real(c_double), intent(in) :: t_in      ! Inlet temperature [K]
    real(c_double) :: t_out                 ! Outlet temperature [K]

    real(c_double) :: gamma, gamma_m1_gamma
    gamma = 1.4_c_double
    gamma_m1_gamma = (gamma - 1.0_c_double) / gamma

    ! T_out = T_in * [1 + (pi_c^((gamma-1)/gamma) - 1) / eta_c]
    t_out = t_in * (1.0_c_double + (pi_c**gamma_m1_gamma - 1.0_c_double) / max(eta_c, 1.0e-6_c_double))
  end function

  ! Advanced compressor model: includes polytropic effects
  pure function compressor_advanced(pi_c, eta_c, t_in, n_stages) result(t_out)
    real(c_double), intent(in) :: pi_c      ! Overall pressure ratio
    real(c_double), intent(in) :: eta_c     ! Polytropic efficiency
    real(c_double), intent(in) :: t_in      ! Inlet temperature [K]
    integer, intent(in) :: n_stages         ! Number of stages (assumed 8)
    real(c_double) :: t_out                 ! Outlet temperature [K]

    real(c_double) :: gamma, n_poly, stage_pr, t_stage
    integer :: i

    gamma = 1.4_c_double
    n_poly = gamma / (gamma - 1.0_c_double) * eta_c
    stage_pr = pi_c ** (1.0_c_double / real(n_stages, c_double))

    t_stage = t_in
    do i = 1, n_stages
      t_stage = t_stage * stage_pr ** ((gamma - 1.0_c_double) / (gamma * max(eta_c, 1.0e-6_c_double)))
    end do
    t_out = t_stage
  end function

  ! Standard turbine model: simple isentropic efficiency
  pure function turbine_standard(pi_t, eta_t, t_in) result(t_out)
    real(c_double), intent(in) :: pi_t      ! Expansion ratio (>1)
    real(c_double), intent(in) :: eta_t     ! Isentropic efficiency
    real(c_double), intent(in) :: t_in      ! Inlet temperature [K]
    real(c_double) :: t_out                 ! Outlet temperature [K]

    real(c_double) :: gamma, gamma_m1_gamma
    gamma = 1.33_c_double  ! Hot gas gamma
    gamma_m1_gamma = (gamma - 1.0_c_double) / gamma

    ! T_out = T_in * [1 - eta_t * (1 - pi_t^(-(gamma-1)/gamma))]
    t_out = t_in * (1.0_c_double - eta_t * (1.0_c_double - pi_t**(-gamma_m1_gamma)))
  end function

  ! Advanced turbine model: cooled turbine with metal temperature constraint
  pure function turbine_advanced(pi_t, eta_t, t_in, t_metal_max) result(t_out)
    real(c_double), intent(in) :: pi_t        ! Expansion ratio
    real(c_double), intent(in) :: eta_t       ! Isentropic efficiency
    real(c_double), intent(in) :: t_in        ! Gas inlet temperature [K]
    real(c_double), intent(in) :: t_metal_max ! Max metal temperature [K]
    real(c_double) :: t_out                   ! Outlet temperature [K]

    real(c_double) :: gamma, gamma_m1_gamma, cooling_factor
    gamma = 1.33_c_double
    gamma_m1_gamma = (gamma - 1.0_c_double) / gamma

    ! Cooling effectiveness factor (simplified model)
    cooling_factor = 1.0_c_double
    if (t_in > t_metal_max) then
      cooling_factor = 0.98_c_double  ! Small penalty for cooling air bleed
    end if

    t_out = t_in * (1.0_c_double - eta_t * cooling_factor * (1.0_c_double - pi_t**(-gamma_m1_gamma)))
  end function

  ! Standard nozzle model: simple velocity coefficient
  pure function nozzle_standard(p_ratio, eta_n, t_in) result(v_exit)
    real(c_double), intent(in) :: p_ratio   ! Nozzle pressure ratio
    real(c_double), intent(in) :: eta_n     ! Velocity coefficient
    real(c_double), intent(in) :: t_in      ! Inlet total temperature [K]
    real(c_double) :: v_exit                ! Exit velocity proxy

    real(c_double) :: gamma, cp, v_ideal
    gamma = 1.33_c_double
    cp = 1150.0_c_double  ! J/(kg*K) for hot gas

    ! Ideal exit velocity (simplified)
    v_ideal = sqrt(2.0_c_double * cp * t_in * (1.0_c_double - (1.0_c_double/p_ratio)**((gamma-1.0_c_double)/gamma)))
    v_exit = eta_n * v_ideal
  end function

  ! Advanced nozzle model: includes divergence losses
  pure function nozzle_advanced(p_ratio, eta_n, t_in, area_ratio) result(v_exit)
    real(c_double), intent(in) :: p_ratio    ! Nozzle pressure ratio
    real(c_double), intent(in) :: eta_n      ! Velocity coefficient
    real(c_double), intent(in) :: t_in       ! Inlet total temperature [K]
    real(c_double), intent(in) :: area_ratio ! Exit/throat area ratio
    real(c_double) :: v_exit                 ! Exit velocity proxy

    real(c_double) :: gamma, cp, v_ideal, divergence_factor
    gamma = 1.33_c_double
    cp = 1150.0_c_double

    ! Divergence loss factor (simplified conical nozzle)
    divergence_factor = 0.5_c_double * (1.0_c_double + cos(0.26_c_double))  ! ~15 deg half-angle

    v_ideal = sqrt(2.0_c_double * cp * t_in * (1.0_c_double - (1.0_c_double/p_ratio)**((gamma-1.0_c_double)/gamma)))
    v_exit = eta_n * divergence_factor * v_ideal
  end function

  ! Compute effective efficiency with degradation
  pure function apply_degradation_eta(eta_base, factor) result(eta_eff)
    real(c_double), intent(in) :: eta_base
    real(c_double), intent(in) :: factor
    real(c_double) :: eta_eff
    eta_eff = max(0.5_c_double, min(1.0_c_double, eta_base * factor))
  end function

  ! Compute effective loss with degradation
  pure function apply_degradation_loss(loss_base, adder) result(loss_eff)
    real(c_double), intent(in) :: loss_base
    real(c_double), intent(in) :: adder
    real(c_double) :: loss_eff
    loss_eff = max(0.0_c_double, min(0.5_c_double, loss_base + adder))
  end function

end module components
