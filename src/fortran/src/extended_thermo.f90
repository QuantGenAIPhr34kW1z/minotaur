!> Extended Thermodynamics Module (v2.1)
!>
!> Implements:
!>   - Variable specific heats Cp(T)
!>   - Real gas corrections (compressibility)
!>   - Advanced combustion equilibrium
!>
!> References:
!>   - NIST-JANAF Thermochemical Tables
!>   - McBride & Gordon, "Computer Program for Complex Chemical Equilibrium" (NASA RP-1311)
!>
!> Author: CSTNSystems, EINIXSA
!> License: LicenseRef-EINIXSA-Internal-Eval

module extended_thermo
    use, intrinsic :: iso_c_binding
    implicit none
    private

    ! Public interfaces
    public :: cp_air, cp_products
    public :: gamma_air, gamma_products
    public :: enthalpy_air, enthalpy_products
    public :: real_gas_factor
    public :: equilibrium_combustion
    public :: kinetic_combustion

    ! Universal gas constant [J/(mol·K)]
    real(c_double), parameter :: R_UNIVERSAL = 8.31446_c_double

    ! Molecular weights [kg/mol]
    real(c_double), parameter :: MW_AIR = 0.02897_c_double
    real(c_double), parameter :: MW_FUEL = 0.170_c_double    ! Approximate kerosene
    real(c_double), parameter :: MW_PRODUCTS = 0.0285_c_double

    ! Reference temperature [K]
    real(c_double), parameter :: T_REF = 298.15_c_double

    ! Fuel heating value [J/kg]
    real(c_double), parameter :: LHV = 43.0e6_c_double

contains

    !---------------------------------------------------------------------------
    !> Variable specific heat of air Cp(T)
    !>
    !> NASA polynomial fit for air (200-1000 K and 1000-6000 K ranges)
    !>
    !> Returns Cp in J/(kg·K)
    !---------------------------------------------------------------------------
    pure function cp_air(T) result(cp)
        real(c_double), intent(in) :: T
        real(c_double) :: cp
        real(c_double) :: a1, a2, a3, a4, a5

        if (T < 1000.0_c_double) then
            ! Low temperature range (200-1000 K)
            a1 =  3.56839620_c_double
            a2 = -6.78729429e-4_c_double
            a3 =  1.55371476e-6_c_double
            a4 = -3.29937060e-12_c_double
            a5 = -4.66395387e-13_c_double
        else
            ! High temperature range (1000-6000 K)
            a1 =  3.08792717_c_double
            a2 =  1.24597184e-3_c_double
            a3 = -4.23718945e-7_c_double
            a4 =  6.74774789e-11_c_double
            a5 = -3.97076972e-15_c_double
        end if

        ! Cp/R = a1 + a2*T + a3*T^2 + a4*T^3 + a5*T^4
        cp = (a1 + a2*T + a3*T**2 + a4*T**3 + a5*T**4) * R_UNIVERSAL / MW_AIR

    end function cp_air

    !---------------------------------------------------------------------------
    !> Variable specific heat of combustion products Cp(T)
    !>
    !> Approximate mixture of CO2, H2O, N2 for kerosene combustion
    !---------------------------------------------------------------------------
    pure function cp_products(T, phi) result(cp)
        real(c_double), intent(in) :: T    ! Temperature [K]
        real(c_double), intent(in) :: phi  ! Equivalence ratio
        real(c_double) :: cp
        real(c_double) :: a1, a2, a3, a4, a5
        real(c_double) :: cp_co2, cp_h2o, cp_n2
        real(c_double) :: x_co2, x_h2o, x_n2

        ! Mole fractions (approximate for phi ~ 1)
        x_co2 = 0.12_c_double * min(phi, 1.0_c_double)
        x_h2o = 0.12_c_double * min(phi, 1.0_c_double)
        x_n2 = 1.0_c_double - x_co2 - x_h2o

        ! CO2 Cp polynomial
        if (T < 1000.0_c_double) then
            a1 =  2.35677352_c_double
            a2 =  8.98459677e-3_c_double
            a3 = -7.12356269e-6_c_double
            a4 =  2.45919022e-9_c_double
            a5 = -1.43699548e-13_c_double
        else
            a1 =  4.63659493_c_double
            a2 =  2.74131991e-3_c_double
            a3 = -9.95828531e-7_c_double
            a4 =  1.60373011e-10_c_double
            a5 = -9.16103468e-15_c_double
        end if
        cp_co2 = (a1 + a2*T + a3*T**2 + a4*T**3 + a5*T**4) * R_UNIVERSAL / 0.04401_c_double

        ! H2O Cp polynomial
        if (T < 1000.0_c_double) then
            a1 =  4.19864056_c_double
            a2 = -2.03643410e-3_c_double
            a3 =  6.52040211e-6_c_double
            a4 = -5.48797062e-9_c_double
            a5 =  1.77197817e-12_c_double
        else
            a1 =  2.67703787_c_double
            a2 =  2.97318160e-3_c_double
            a3 = -7.73769690e-7_c_double
            a4 =  9.44336689e-11_c_double
            a5 = -4.26900959e-15_c_double
        end if
        cp_h2o = (a1 + a2*T + a3*T**2 + a4*T**3 + a5*T**4) * R_UNIVERSAL / 0.01802_c_double

        ! N2 Cp polynomial
        if (T < 1000.0_c_double) then
            a1 =  3.53100528_c_double
            a2 = -1.23660988e-4_c_double
            a3 = -5.02999433e-7_c_double
            a4 =  2.43530612e-9_c_double
            a5 = -1.40881235e-12_c_double
        else
            a1 =  2.95257637_c_double
            a2 =  1.39690040e-3_c_double
            a3 = -4.92631603e-7_c_double
            a4 =  7.86010195e-11_c_double
            a5 = -4.60755204e-15_c_double
        end if
        cp_n2 = (a1 + a2*T + a3*T**2 + a4*T**3 + a5*T**4) * R_UNIVERSAL / 0.02801_c_double

        ! Mass-weighted average
        cp = x_co2 * cp_co2 + x_h2o * cp_h2o + x_n2 * cp_n2

    end function cp_products

    !---------------------------------------------------------------------------
    !> Ratio of specific heats for air gamma(T)
    !---------------------------------------------------------------------------
    pure function gamma_air(T) result(gam)
        real(c_double), intent(in) :: T
        real(c_double) :: gam
        real(c_double) :: cp, cv, R_specific

        R_specific = R_UNIVERSAL / MW_AIR
        cp = cp_air(T)
        cv = cp - R_specific
        gam = cp / cv

    end function gamma_air

    !---------------------------------------------------------------------------
    !> Ratio of specific heats for products gamma(T)
    !---------------------------------------------------------------------------
    pure function gamma_products(T, phi) result(gam)
        real(c_double), intent(in) :: T
        real(c_double), intent(in) :: phi
        real(c_double) :: gam
        real(c_double) :: cp, cv, R_specific

        R_specific = R_UNIVERSAL / MW_PRODUCTS
        cp = cp_products(T, phi)
        cv = cp - R_specific
        gam = cp / cv

    end function gamma_products

    !---------------------------------------------------------------------------
    !> Specific enthalpy of air h(T) [J/kg]
    !>
    !> Integrated from Cp: h = ∫Cp dT + h_ref
    !---------------------------------------------------------------------------
    pure function enthalpy_air(T) result(h)
        real(c_double), intent(in) :: T
        real(c_double) :: h
        real(c_double) :: a1, a2, a3, a4, a5, a6

        if (T < 1000.0_c_double) then
            a1 =  3.56839620_c_double
            a2 = -6.78729429e-4_c_double
            a3 =  1.55371476e-6_c_double
            a4 = -3.29937060e-12_c_double
            a5 = -4.66395387e-13_c_double
            a6 = -1.06263943e3_c_double
        else
            a1 =  3.08792717_c_double
            a2 =  1.24597184e-3_c_double
            a3 = -4.23718945e-7_c_double
            a4 =  6.74774789e-11_c_double
            a5 = -3.97076972e-15_c_double
            a6 = -9.95262755e2_c_double
        end if

        ! H/RT = a1 + a2*T/2 + a3*T^2/3 + a4*T^3/4 + a5*T^4/5 + a6/T
        h = (a1 + a2*T/2.0_c_double + a3*T**2/3.0_c_double + a4*T**3/4.0_c_double &
             + a5*T**4/5.0_c_double + a6/T) * R_UNIVERSAL * T / MW_AIR

    end function enthalpy_air

    !---------------------------------------------------------------------------
    !> Specific enthalpy of products h(T) [J/kg]
    !---------------------------------------------------------------------------
    pure function enthalpy_products(T, phi) result(h)
        real(c_double), intent(in) :: T
        real(c_double), intent(in) :: phi
        real(c_double) :: h
        real(c_double) :: cp_avg, dT

        ! Simple trapezoidal integration from T_ref
        cp_avg = 0.5_c_double * (cp_products(T_REF, phi) + cp_products(T, phi))
        dT = T - T_REF
        h = cp_avg * dT

    end function enthalpy_products

    !---------------------------------------------------------------------------
    !> Real gas compressibility factor Z(T, P)
    !>
    !> Peng-Robinson equation of state for air
    !>
    !> Returns Z such that PV = ZnRT
    !---------------------------------------------------------------------------
    pure function real_gas_factor(T, P) result(Z)
        real(c_double), intent(in) :: T  ! Temperature [K]
        real(c_double), intent(in) :: P  ! Pressure [Pa]
        real(c_double) :: Z

        ! Critical properties for air
        real(c_double), parameter :: T_c = 132.5_c_double  ! K
        real(c_double), parameter :: P_c = 3.77e6_c_double  ! Pa
        real(c_double), parameter :: omega = 0.0335_c_double  ! Acentric factor

        real(c_double) :: T_r, P_r, kappa, alpha, a, b
        real(c_double) :: A, B, Z0, Z1, Z2

        ! Reduced properties
        T_r = T / T_c
        P_r = P / P_c

        ! Peng-Robinson parameters
        kappa = 0.37464_c_double + 1.54226_c_double*omega - 0.26992_c_double*omega**2
        alpha = (1.0_c_double + kappa*(1.0_c_double - sqrt(T_r)))**2

        a = 0.45724_c_double * (R_UNIVERSAL * T_c)**2 / P_c * alpha
        b = 0.07780_c_double * R_UNIVERSAL * T_c / P_c

        A = a * P / (R_UNIVERSAL * T)**2
        B = b * P / (R_UNIVERSAL * T)

        ! Solve cubic: Z^3 - (1-B)Z^2 + (A-3B^2-2B)Z - (AB-B^2-B^3) = 0
        ! Using approximation for gas phase (largest root)
        Z0 = 1.0_c_double
        Z1 = Z0 - (Z0**3 - (1.0_c_double-B)*Z0**2 + (A-3.0_c_double*B**2-2.0_c_double*B)*Z0 &
                   - (A*B-B**2-B**3)) / &
                  (3.0_c_double*Z0**2 - 2.0_c_double*(1.0_c_double-B)*Z0 + (A-3.0_c_double*B**2-2.0_c_double*B))
        Z2 = Z1 - (Z1**3 - (1.0_c_double-B)*Z1**2 + (A-3.0_c_double*B**2-2.0_c_double*B)*Z1 &
                   - (A*B-B**2-B**3)) / &
                  (3.0_c_double*Z1**2 - 2.0_c_double*(1.0_c_double-B)*Z1 + (A-3.0_c_double*B**2-2.0_c_double*B))

        Z = max(Z2, 0.8_c_double)  ! Ensure physical value

    end function real_gas_factor

    !---------------------------------------------------------------------------
    !> Equilibrium combustion temperature
    !>
    !> Solves energy balance: h_reactants = h_products + heat_loss
    !>
    !> Returns adiabatic flame temperature [K]
    !---------------------------------------------------------------------------
    pure function equilibrium_combustion(T_inlet, phi, eta_comb) result(T_flame)
        real(c_double), intent(in) :: T_inlet   ! Inlet temperature [K]
        real(c_double), intent(in) :: phi       ! Equivalence ratio
        real(c_double), intent(in) :: eta_comb  ! Combustion efficiency
        real(c_double) :: T_flame

        real(c_double) :: h_reactants, h_fuel, q_released
        real(c_double) :: T_guess, T_new, h_prod, err
        integer :: iter

        ! Enthalpy of reactants (air)
        h_reactants = enthalpy_air(T_inlet)

        ! Heat released from fuel
        q_released = eta_comb * LHV * phi / (1.0_c_double + phi * 15.0_c_double)

        ! Initial guess
        T_guess = T_inlet + q_released / cp_products(T_inlet + 500.0_c_double, phi)

        ! Newton iteration
        do iter = 1, 20
            h_prod = enthalpy_products(T_guess, phi)
            err = h_reactants + q_released - h_prod

            ! Update temperature
            T_new = T_guess + err / cp_products(T_guess, phi)
            T_new = max(T_inlet, min(T_new, 3000.0_c_double))

            if (abs(T_new - T_guess) < 0.1_c_double) exit
            T_guess = T_new
        end do

        T_flame = T_new

    end function equilibrium_combustion

    !---------------------------------------------------------------------------
    !> Kinetic combustion model with finite-rate chemistry
    !>
    !> Single-step global reaction: Fuel + Air → Products
    !> Arrhenius rate: k = A * exp(-Ea/RT)
    !>
    !> Returns effective reaction temperature [K]
    !---------------------------------------------------------------------------
    pure function kinetic_combustion(T_inlet, phi, residence_time, P) result(T_exit)
        real(c_double), intent(in) :: T_inlet        ! Inlet temperature [K]
        real(c_double), intent(in) :: phi            ! Equivalence ratio
        real(c_double), intent(in) :: residence_time ! Combustor residence time [s]
        real(c_double), intent(in) :: P              ! Pressure [Pa]
        real(c_double) :: T_exit

        ! Arrhenius parameters (global one-step)
        real(c_double), parameter :: A_pre = 1.0e12_c_double    ! Pre-exponential [1/s]
        real(c_double), parameter :: E_a = 1.5e5_c_double       ! Activation energy [J/mol]

        real(c_double) :: k_rate, conversion, T_ad, dT

        ! Reaction rate
        k_rate = A_pre * exp(-E_a / (R_UNIVERSAL * T_inlet))

        ! Conversion fraction (first-order kinetics)
        conversion = 1.0_c_double - exp(-k_rate * residence_time)
        conversion = min(conversion, 0.99_c_double)  ! Cap at 99%

        ! Adiabatic flame temperature
        T_ad = equilibrium_combustion(T_inlet, phi, 1.0_c_double)

        ! Temperature rise proportional to conversion
        dT = (T_ad - T_inlet) * conversion
        T_exit = T_inlet + dT

    end function kinetic_combustion

end module extended_thermo
