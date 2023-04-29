module minotaur_api
  use, intrinsic :: iso_c_binding
  use minotaur_types
  use cycle_solver
  use cycle_solver_ad
  use extended_thermo
  use physics_options
  implicit none
contains

  ! Standard solve (backward compatible)
  subroutine minotaur_solve_c(inp, out) bind(C, name="minotaur_solve_c")
    type(MinotaurInput), intent(in) :: inp
    type(MinotaurOutput), intent(out) :: out
    call solve_cycle(inp, out)
  end subroutine

  ! Extended solve with component models and degradation (v2.4)
  subroutine minotaur_solve_ext_c(inp_ext, out) bind(C, name="minotaur_solve_ext_c")
    type(MinotaurInputExt), intent(in) :: inp_ext
    type(MinotaurOutput), intent(out) :: out
    type(MinotaurDiagnostics) :: diag
    call solve_cycle_extended(inp_ext, out, diag)
  end subroutine

  ! Get version information (v2.1)
  subroutine minotaur_get_version(major, minor, patch) bind(C, name="minotaur_get_version")
    integer(c_int), intent(out) :: major, minor, patch
    major = 1
    minor = 1
    patch = 0
  end subroutine

  ! Get schema version (v2.5)
  subroutine minotaur_get_schema_version(major, minor, patch) bind(C, name="minotaur_get_schema_version")
    integer(c_int), intent(out) :: major, minor, patch
    major = 1
    minor = 0
    patch = 0
  end subroutine

  !---------------------------------------------------------------------------
  ! Automatic Differentiation API (v2.8)
  !---------------------------------------------------------------------------

  ! Compute exact gradient w.r.t. single parameter via forward-mode AD
  subroutine minotaur_solve_ad_c(mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz, &
                                  fuel_k, t4_max, seed_param, &
                                  tsfc_val, tsfc_der, thrust_val, thrust_der, &
                                  t4_val, t4_der, status) bind(C, name="minotaur_solve_ad_c")
    real(c_double), intent(in), value :: mach, alt_km, bpr, opr
    real(c_double), intent(in), value :: eta_comp, eta_turb, eta_nozz, fuel_k, t4_max
    integer(c_int), intent(in), value :: seed_param
    real(c_double), intent(out) :: tsfc_val, tsfc_der
    real(c_double), intent(out) :: thrust_val, thrust_der
    real(c_double), intent(out) :: t4_val, t4_der
    integer(c_int), intent(out) :: status

    call solve_cycle_ad(mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz, &
                        fuel_k, t4_max, seed_param, &
                        tsfc_val, tsfc_der, thrust_val, thrust_der, t4_val, t4_der, status)
  end subroutine

  ! Compute full Jacobian matrix (6 params x 3 outputs)
  subroutine minotaur_jacobian_c(mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max, &
                                  jacobian, base_tsfc, base_thrust, base_t4, status) &
                                  bind(C, name="minotaur_jacobian_c")
    real(c_double), intent(in), value :: mach, alt_km, bpr, opr
    real(c_double), intent(in), value :: eta_comp, eta_turb, t4_max
    real(c_double), intent(out) :: jacobian(6, 3)
    real(c_double), intent(out) :: base_tsfc, base_thrust, base_t4
    integer(c_int), intent(out) :: status

    call compute_jacobian(mach, alt_km, bpr, opr, eta_comp, eta_turb, t4_max, &
                          jacobian, base_tsfc, base_thrust, base_t4, status)
  end subroutine

  !---------------------------------------------------------------------------
  ! Extended Physics API (v2.1)
  !---------------------------------------------------------------------------

  ! Get variable specific heat of air Cp(T) [J/(kg·K)]
  subroutine minotaur_cp_air_c(T, cp) bind(C, name="minotaur_cp_air_c")
    real(c_double), intent(in), value :: T
    real(c_double), intent(out) :: cp
    cp = cp_air(T)
  end subroutine

  ! Get variable specific heat of products Cp(T, phi) [J/(kg·K)]
  subroutine minotaur_cp_products_c(T, phi, cp) bind(C, name="minotaur_cp_products_c")
    real(c_double), intent(in), value :: T, phi
    real(c_double), intent(out) :: cp
    cp = cp_products(T, phi)
  end subroutine

  ! Get gamma for air at temperature T
  subroutine minotaur_gamma_air_c(T, gam) bind(C, name="minotaur_gamma_air_c")
    real(c_double), intent(in), value :: T
    real(c_double), intent(out) :: gam
    gam = gamma_air(T)
  end subroutine

  ! Get real gas compressibility factor Z(T, P)
  subroutine minotaur_real_gas_z_c(T, P, Z) bind(C, name="minotaur_real_gas_z_c")
    real(c_double), intent(in), value :: T, P
    real(c_double), intent(out) :: Z
    Z = real_gas_factor(T, P)
  end subroutine

  ! Compute equilibrium combustion temperature [K]
  subroutine minotaur_combustion_eq_c(T_inlet, phi, eta_comb, T_flame) &
              bind(C, name="minotaur_combustion_eq_c")
    real(c_double), intent(in), value :: T_inlet, phi, eta_comb
    real(c_double), intent(out) :: T_flame
    T_flame = equilibrium_combustion(T_inlet, phi, eta_comb)
  end subroutine

  ! Compute kinetic combustion temperature [K]
  subroutine minotaur_combustion_kin_c(T_inlet, phi, tau, P, T_exit) &
              bind(C, name="minotaur_combustion_kin_c")
    real(c_double), intent(in), value :: T_inlet, phi, tau, P
    real(c_double), intent(out) :: T_exit
    T_exit = kinetic_combustion(T_inlet, phi, tau, P)
  end subroutine

  ! Get default physics configuration
  subroutine minotaur_physics_defaults_c(cfg) bind(C, name="minotaur_physics_defaults_c")
    type(PhysicsConfig), intent(out) :: cfg
    cfg = physics_defaults()
  end subroutine

end module
