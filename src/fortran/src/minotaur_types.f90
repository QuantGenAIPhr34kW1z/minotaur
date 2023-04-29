module minotaur_types
  use, intrinsic :: iso_c_binding
  implicit none

  ! Status codes
  integer, parameter :: STATUS_OK = 0
  integer, parameter :: STATUS_MAXITER = 1
  integer, parameter :: STATUS_DIVERGED = 2
  integer, parameter :: STATUS_INVARIANT_VIOL = 3
  integer, parameter :: STATUS_CONSTRAINT_VIOL = 4
  integer, parameter :: STATUS_NONPHYSICAL = 5

  ! Maximum iterations for history tracking
  integer, parameter :: MAX_HISTORY = 256

  ! Schema and solver versions (v2.5)
  character(len=*), parameter :: SCHEMA_VERSION = "1.0.0"
  character(len=*), parameter :: SOLVER_VERSION = "0.5.0"
  character(len=*), parameter :: CSTNSystems_PROGRAM_ID = "CSTNSystems-MINOTAUR"

  ! Component model identifiers (v2.4)
  integer, parameter :: MODEL_STANDARD = 0
  integer, parameter :: MODEL_ADVANCED = 1

  type, bind(C) :: MinotaurInput
     real(c_double) :: mach
     real(c_double) :: alt_km
     real(c_double) :: bpr
     real(c_double) :: opr
     real(c_double) :: eta_comp
     real(c_double) :: eta_turb
     real(c_double) :: eta_nozz
     real(c_double) :: fuel_k
     integer(c_int)  :: max_iter
     real(c_double)  :: tol
     real(c_double)  :: damping
     real(c_double)  :: mass_tol
     real(c_double)  :: energy_tol
     real(c_double)  :: t4_max
  end type

  ! Extended input with component models (v2.4)
  type, bind(C) :: MinotaurInputExt
     ! Base parameters (same as MinotaurInput)
     real(c_double) :: mach
     real(c_double) :: alt_km
     real(c_double) :: bpr
     real(c_double) :: opr
     real(c_double) :: eta_comp
     real(c_double) :: eta_turb
     real(c_double) :: eta_nozz
     real(c_double) :: fuel_k
     integer(c_int)  :: max_iter
     real(c_double)  :: tol
     real(c_double)  :: damping
     real(c_double)  :: mass_tol
     real(c_double)  :: energy_tol
     real(c_double)  :: t4_max
     ! Component model selection
     integer(c_int)  :: compressor_model   ! 0=standard, 1=advanced
     integer(c_int)  :: turbine_model      ! 0=standard, 1=advanced
     integer(c_int)  :: nozzle_model       ! 0=standard, 1=advanced
     ! Loss coefficients
     real(c_double)  :: inlet_loss         ! Inlet pressure loss coefficient
     real(c_double)  :: burner_loss        ! Combustor pressure loss
     real(c_double)  :: turbine_mech_loss  ! Turbine mechanical loss
     real(c_double)  :: nozzle_loss        ! Nozzle velocity loss coefficient
     ! Degradation factors
     real(c_double)  :: eta_comp_factor    ! Compressor efficiency multiplier (1.0 = nominal)
     real(c_double)  :: eta_turb_factor    ! Turbine efficiency multiplier
     real(c_double)  :: loss_adder         ! Additional pressure loss
     integer(c_int)  :: is_degraded        ! 1 if degradation scenario, 0 otherwise
  end type

  type, bind(C) :: MinotaurOutput
     integer(c_int) :: status
     integer(c_int) :: iter
     real(c_double) :: mass_resid
     real(c_double) :: energy_resid
     real(c_double) :: t4
     real(c_double) :: tsfc_proxy
     real(c_double) :: thrust_proxy
     real(c_double) :: final_bpr
     real(c_double) :: final_residual
  end type

  ! Extended output with convergence history
  type, bind(C) :: ConvergenceRecord
     integer(c_int) :: iteration
     real(c_double) :: residual_norm
     real(c_double) :: bpr
     real(c_double) :: t4
     real(c_double) :: step_size
     integer(c_int) :: admissible  ! 1=true, 0=false
  end type

  type, bind(C) :: MinotaurDiagnostics
     integer(c_int) :: history_len
     type(ConvergenceRecord) :: history(MAX_HISTORY)
     real(c_double) :: last_admissible_bpr
     real(c_double) :: last_admissible_t4
     integer(c_int) :: line_search_steps
     real(c_double) :: initial_residual
     real(c_double) :: best_residual
  end type

contains
  pure logical function is_finite(x)
    real(c_double), intent(in) :: x
    is_finite = (x == x) .and. (abs(x) < huge(x))
  end function

  pure function status_name(code) result(name)
    integer, intent(in) :: code
    character(len=16) :: name
    select case(code)
      case(STATUS_OK)
        name = "OK"
      case(STATUS_MAXITER)
        name = "MAXITER"
      case(STATUS_DIVERGED)
        name = "DIVERGED"
      case(STATUS_INVARIANT_VIOL)
        name = "INVARIANT_VIOL"
      case(STATUS_CONSTRAINT_VIOL)
        name = "CONSTRAINT_VIOL"
      case(STATUS_NONPHYSICAL)
        name = "NONPHYSICAL"
      case default
        name = "UNKNOWN"
    end select
  end function
end module
