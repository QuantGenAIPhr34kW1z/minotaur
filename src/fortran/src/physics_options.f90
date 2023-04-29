!> Physics Options Module (v2.1)
!>
!> Defines physics model options and provides unified interface
!> for selecting between simple and extended thermodynamic models.
!>
!> Author: CSTNSystems, EINIXSA
!> License: LicenseRef-EINIXSA-Internal-Eval

module physics_options
    use, intrinsic :: iso_c_binding
    implicit none
    private

    public :: PhysicsConfig, physics_defaults
    public :: COMBUSTION_SIMPLE, COMBUSTION_EQUILIBRIUM, COMBUSTION_KINETIC

    ! Combustion model identifiers
    integer(c_int), parameter :: COMBUSTION_SIMPLE = 0
    integer(c_int), parameter :: COMBUSTION_EQUILIBRIUM = 1
    integer(c_int), parameter :: COMBUSTION_KINETIC = 2

    !> Physics configuration type
    type, bind(C) :: PhysicsConfig
        ! Thermodynamic options
        integer(c_int)  :: variable_cp      ! 0=constant Cp, 1=Cp(T)
        integer(c_int)  :: real_gas         ! 0=ideal gas, 1=real gas (Z factor)

        ! Combustion options
        integer(c_int)  :: combustion_model ! 0=simple, 1=equilibrium, 2=kinetic

        ! Model parameters
        real(c_double)  :: combustion_efficiency  ! η_comb [0.9, 1.0]
        real(c_double)  :: residence_time         ! τ [s] for kinetic model
        real(c_double)  :: dissociation_factor    ! Dissociation at high T

        ! Reference values for non-dimensionalization
        real(c_double)  :: cp_ref            ! Reference Cp [J/(kg·K)]
        real(c_double)  :: gamma_ref         ! Reference gamma
    end type PhysicsConfig

contains

    !---------------------------------------------------------------------------
    !> Create default physics configuration
    !---------------------------------------------------------------------------
    pure function physics_defaults() result(cfg)
        type(PhysicsConfig) :: cfg

        cfg%variable_cp = 0           ! Constant Cp (classic model)
        cfg%real_gas = 0              ! Ideal gas
        cfg%combustion_model = COMBUSTION_SIMPLE

        cfg%combustion_efficiency = 0.99_c_double
        cfg%residence_time = 0.002_c_double  ! 2 ms typical
        cfg%dissociation_factor = 0.0_c_double

        cfg%cp_ref = 1004.5_c_double  ! Air at 300 K
        cfg%gamma_ref = 1.4_c_double

    end function physics_defaults

end module physics_options
