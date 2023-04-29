!> Automatic Differentiation Enabled Cycle Solver
!>
!> Forward-mode AD for exact gradient computation via dual numbers.
!> Computes Jacobian of outputs (TSFC, Thrust, T4) w.r.t. inputs.
!>
!> Author: CSTNSystems, EINIXSA
!> License: LicenseRef-EINIXSA-Internal-Eval

module cycle_solver_ad
    use, intrinsic :: iso_c_binding
    use dual_numbers
    use minotaur_types
    implicit none
    private

    public :: solve_cycle_ad, compute_jacobian

    !> Physical constants
    real(c_double), parameter :: GAMMA = 1.4_c_double
    real(c_double), parameter :: R_GAS = 287.05_c_double
    real(c_double), parameter :: CP = 1004.5_c_double

contains

    !---------------------------------------------------------------------------
    !> Atmospheric model using dual numbers
    !---------------------------------------------------------------------------
    pure subroutine atmosphere_dual(alt_km, t_amb, p_amb)
        type(dual), intent(in) :: alt_km
        type(dual), intent(out) :: t_amb, p_amb

        type(dual) :: alt_m, t_ratio, p_ratio

        ! Convert km to m
        alt_m = alt_km * 1000.0_c_double

        ! Temperature lapse rate model (troposphere approximation)
        t_amb = dual_const(288.15_c_double) - alt_m * 0.0065_c_double

        ! Pressure ratio (barometric formula)
        t_ratio = t_amb / 288.15_c_double
        p_ratio = t_ratio ** 5.256_c_double
        p_amb = dual_const(101325.0_c_double) * p_ratio

    end subroutine atmosphere_dual

    !---------------------------------------------------------------------------
    !> Isentropic relations with dual numbers
    !---------------------------------------------------------------------------
    pure function isentropic_t_ratio(pi, gamma_val) result(tau)
        type(dual), intent(in) :: pi
        real(c_double), intent(in) :: gamma_val
        type(dual) :: tau
        real(c_double) :: exponent

        exponent = (gamma_val - 1.0_c_double) / gamma_val
        tau = pi ** exponent
    end function

    !---------------------------------------------------------------------------
    !> Core cycle calculation with dual number inputs
    !---------------------------------------------------------------------------
    pure subroutine cycle_core_dual(mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz, &
                                     fuel_k, t4_max, tsfc, thrust, t4, converged)
        type(dual), intent(in) :: mach, alt_km, bpr, opr
        type(dual), intent(in) :: eta_comp, eta_turb, eta_nozz, fuel_k
        real(c_double), intent(in) :: t4_max
        type(dual), intent(out) :: tsfc, thrust, t4
        logical, intent(out) :: converged

        ! Intermediate dual variables
        type(dual) :: t_amb, p_amb
        type(dual) :: v_flight, t0, p0
        type(dual) :: pi_c, tau_c, t2, t3
        type(dual) :: pi_t, tau_t, t5
        type(dual) :: v_core, v_bypass
        type(dual) :: f_ratio, mass_core, mass_bypass
        type(dual) :: f_core, f_bypass
        type(dual) :: a_sound

        ! Get ambient conditions
        call atmosphere_dual(alt_km, t_amb, p_amb)

        ! Speed of sound and flight velocity
        a_sound = sqrt_d(dual_const(GAMMA * R_GAS) * t_amb)
        v_flight = mach * a_sound

        ! Stagnation conditions
        t0 = t_amb * (dual_const(1.0_c_double) + dual_const((GAMMA - 1.0_c_double) / 2.0_c_double) * mach * mach)
        p0 = p_amb * ((t0 / t_amb) ** (GAMMA / (GAMMA - 1.0_c_double)))

        ! Compressor
        pi_c = opr  ! Overall pressure ratio
        tau_c = isentropic_t_ratio(pi_c, GAMMA)
        t2 = t0
        t3 = t2 + (t2 * (tau_c - dual_const(1.0_c_double))) / eta_comp

        ! Combustor: T4 estimation via fuel parameter
        t4 = t3 + fuel_k * dual_const(400.0_c_double)

        ! Check T4 constraint
        if (t4%val > t4_max) then
            converged = .false.
            tsfc = dual_const(0.0_c_double)
            thrust = dual_const(0.0_c_double)
            return
        end if

        ! Turbine: Work balance with compressor
        ! tau_t = 1 - (t3 - t2) / (eta_turb * t4)
        tau_t = dual_const(1.0_c_double) - (t3 - t2) / (eta_turb * t4)
        t5 = t4 * tau_t

        ! Core exhaust velocity (simplified)
        v_core = sqrt_d(dual_const(2.0_c_double * CP) * eta_nozz * (t5 - t_amb))

        ! Bypass exhaust velocity
        v_bypass = sqrt_d(dual_const(2.0_c_double * CP) * eta_nozz * (t3 - t_amb) * dual_const(0.3_c_double))

        ! Mass flow split
        mass_core = dual_const(1.0_c_double) / (bpr + dual_const(1.0_c_double))
        mass_bypass = bpr / (bpr + dual_const(1.0_c_double))

        ! Specific thrust contributions
        f_core = mass_core * (v_core - v_flight)
        f_bypass = mass_bypass * (v_bypass - v_flight)

        ! Total specific thrust
        thrust = f_core + f_bypass

        ! Fuel-air ratio approximation
        f_ratio = (t4 - t3) / dual_const(43000.0_c_double / CP)

        ! TSFC = fuel flow / thrust (proxy)
        if (thrust%val > 0.01_c_double) then
            tsfc = (f_ratio * mass_core) / thrust
            converged = .true.
        else
            tsfc = dual_const(999.0_c_double)
            converged = .false.
        end if

    end subroutine cycle_core_dual

    !---------------------------------------------------------------------------
    !> Solve cycle with single parameter seeded for derivative
    !---------------------------------------------------------------------------
    subroutine solve_cycle_ad(mach, alt_km, bpr, opr, eta_comp, eta_turb, eta_nozz, &
                               fuel_k, t4_max, seed_param, &
                               tsfc_val, tsfc_der, thrust_val, thrust_der, t4_val, t4_der, status)
        real(c_double), intent(in) :: mach, alt_km, bpr, opr
        real(c_double), intent(in) :: eta_comp, eta_turb, eta_nozz, fuel_k, t4_max
        integer(c_int), intent(in) :: seed_param  ! Which parameter to differentiate (1-6)
        real(c_double), intent(out) :: tsfc_val, tsfc_der
        real(c_double), intent(out) :: thrust_val, thrust_der
        real(c_double), intent(out) :: t4_val, t4_der
        integer(c_int), intent(out) :: status

        type(dual) :: d_mach, d_alt, d_bpr, d_opr, d_eta_c, d_eta_t, d_eta_n, d_fuel
        type(dual) :: tsfc, thrust, t4
        logical :: converged

        ! Initialize all as constants
        d_mach = dual_const(mach)
        d_alt = dual_const(alt_km)
        d_bpr = dual_const(bpr)
        d_opr = dual_const(opr)
        d_eta_c = dual_const(eta_comp)
        d_eta_t = dual_const(eta_turb)
        d_eta_n = dual_const(eta_nozz)
        d_fuel = dual_const(fuel_k)

        ! Seed the selected parameter
        select case (seed_param)
            case (1)
                d_mach = dual_var(mach)
            case (2)
                d_alt = dual_var(alt_km)
            case (3)
                d_bpr = dual_var(bpr)
            case (4)
                d_opr = dual_var(opr)
            case (5)
                d_eta_c = dual_var(eta_comp)
            case (6)
                d_eta_t = dual_var(eta_turb)
        end select

        ! Run cycle calculation
        call cycle_core_dual(d_mach, d_alt, d_bpr, d_opr, d_eta_c, d_eta_t, d_eta_n, &
                             d_fuel, t4_max, tsfc, thrust, t4, converged)

        ! Extract values and derivatives
        tsfc_val = tsfc%val
        tsfc_der = tsfc%der
        thrust_val = thrust%val
        thrust_der = thrust%der
        t4_val = t4%val
        t4_der = t4%der

        if (converged) then
            status = STATUS_OK
        else
            status = STATUS_CONSTRAINT_VIOL
        end if

    end subroutine solve_cycle_ad

    !---------------------------------------------------------------------------
    !> Compute full Jacobian matrix
    !---------------------------------------------------------------------------
    subroutine compute_jacobian(mach, alt_km, bpr, opr, eta_comp, eta_turb, &
                                 t4_max, jacobian, base_tsfc, base_thrust, base_t4, status)
        real(c_double), intent(in) :: mach, alt_km, bpr, opr
        real(c_double), intent(in) :: eta_comp, eta_turb, t4_max
        real(c_double), intent(out) :: jacobian(6, 3)  ! 6 params x 3 outputs
        real(c_double), intent(out) :: base_tsfc, base_thrust, base_t4
        integer(c_int), intent(out) :: status

        integer :: i
        real(c_double) :: tsfc_val, tsfc_der, thrust_val, thrust_der, t4_val, t4_der
        integer(c_int) :: local_status

        status = STATUS_OK

        ! Compute derivatives for each parameter
        do i = 1, 6
            call solve_cycle_ad(mach, alt_km, bpr, opr, eta_comp, eta_turb, &
                               0.95_c_double, 1.0_c_double, t4_max, i, &
                               tsfc_val, tsfc_der, thrust_val, thrust_der, t4_val, t4_der, local_status)

            if (local_status /= STATUS_OK) then
                status = local_status
            end if

            ! Store in Jacobian: rows = params, cols = outputs (tsfc, thrust, t4)
            jacobian(i, 1) = tsfc_der
            jacobian(i, 2) = thrust_der
            jacobian(i, 3) = t4_der

            ! Store base values from first run
            if (i == 1) then
                base_tsfc = tsfc_val
                base_thrust = thrust_val
                base_t4 = t4_val
            end if
        end do

    end subroutine compute_jacobian

end module cycle_solver_ad
