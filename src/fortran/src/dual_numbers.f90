!> Dual Numbers for Forward-Mode Automatic Differentiation
!>
!> Dual numbers extend real numbers with an infinitesimal component:
!>   x + ε·dx  where ε² = 0
!>
!> This enables exact gradient computation via operator overloading.
!>
!> References:
!>   - Griewank & Walther, "Evaluating Derivatives" (2008)
!>   - Naumann, "The Art of Differentiating Computer Programs" (2012)

module dual_numbers
    use, intrinsic :: iso_c_binding
    implicit none
    private

    public :: dual, dual_from_real, dual_var, dual_const
    public :: dual_value, dual_deriv
    public :: operator(+), operator(-), operator(*), operator(/)
    public :: operator(**), operator(<), operator(>), operator(<=), operator(>=)
    public :: sqrt_d, exp_d, log_d, sin_d, cos_d, abs_d, max_d, min_d

    !> Dual number type: value + derivative
    type :: dual
        real(c_double) :: val = 0.0_c_double   !< Function value
        real(c_double) :: der = 0.0_c_double   !< Derivative w.r.t. seed
    end type dual

    !> Operator interfaces
    interface operator(+)
        module procedure add_dd, add_dr, add_rd
    end interface

    interface operator(-)
        module procedure sub_dd, sub_dr, sub_rd, neg_d
    end interface

    interface operator(*)
        module procedure mul_dd, mul_dr, mul_rd
    end interface

    interface operator(/)
        module procedure div_dd, div_dr, div_rd
    end interface

    interface operator(**)
        module procedure pow_di, pow_dr, pow_dd
    end interface

    interface operator(<)
        module procedure lt_dd, lt_dr, lt_rd
    end interface

    interface operator(>)
        module procedure gt_dd, gt_dr, gt_rd
    end interface

    interface operator(<=)
        module procedure le_dd, le_dr, le_rd
    end interface

    interface operator(>=)
        module procedure ge_dd, ge_dr, ge_rd
    end interface

contains

    !---------------------------------------------------------------------------
    ! Constructors
    !---------------------------------------------------------------------------

    !> Create dual from real value (constant, zero derivative)
    pure elemental function dual_from_real(x) result(d)
        real(c_double), intent(in) :: x
        type(dual) :: d
        d%val = x
        d%der = 0.0_c_double
    end function

    !> Create independent variable (derivative = 1)
    pure elemental function dual_var(x) result(d)
        real(c_double), intent(in) :: x
        type(dual) :: d
        d%val = x
        d%der = 1.0_c_double
    end function

    !> Create constant (derivative = 0)
    pure elemental function dual_const(x) result(d)
        real(c_double), intent(in) :: x
        type(dual) :: d
        d%val = x
        d%der = 0.0_c_double
    end function

    !> Extract value
    pure elemental function dual_value(d) result(v)
        type(dual), intent(in) :: d
        real(c_double) :: v
        v = d%val
    end function

    !> Extract derivative
    pure elemental function dual_deriv(d) result(v)
        type(dual), intent(in) :: d
        real(c_double) :: v
        v = d%der
    end function

    !---------------------------------------------------------------------------
    ! Addition
    !---------------------------------------------------------------------------

    pure elemental function add_dd(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        c%val = a%val + b%val
        c%der = a%der + b%der
    end function

    pure elemental function add_dr(a, b) result(c)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        type(dual) :: c
        c%val = a%val + b
        c%der = a%der
    end function

    pure elemental function add_rd(a, b) result(c)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        type(dual) :: c
        c%val = a + b%val
        c%der = b%der
    end function

    !---------------------------------------------------------------------------
    ! Subtraction
    !---------------------------------------------------------------------------

    pure elemental function sub_dd(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        c%val = a%val - b%val
        c%der = a%der - b%der
    end function

    pure elemental function sub_dr(a, b) result(c)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        type(dual) :: c
        c%val = a%val - b
        c%der = a%der
    end function

    pure elemental function sub_rd(a, b) result(c)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        type(dual) :: c
        c%val = a - b%val
        c%der = -b%der
    end function

    pure elemental function neg_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = -a%val
        c%der = -a%der
    end function

    !---------------------------------------------------------------------------
    ! Multiplication: d(ab) = a·db + b·da
    !---------------------------------------------------------------------------

    pure elemental function mul_dd(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        c%val = a%val * b%val
        c%der = a%val * b%der + a%der * b%val
    end function

    pure elemental function mul_dr(a, b) result(c)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        type(dual) :: c
        c%val = a%val * b
        c%der = a%der * b
    end function

    pure elemental function mul_rd(a, b) result(c)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        type(dual) :: c
        c%val = a * b%val
        c%der = a * b%der
    end function

    !---------------------------------------------------------------------------
    ! Division: d(a/b) = (b·da - a·db) / b²
    !---------------------------------------------------------------------------

    pure elemental function div_dd(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        c%val = a%val / b%val
        c%der = (a%der * b%val - a%val * b%der) / (b%val * b%val)
    end function

    pure elemental function div_dr(a, b) result(c)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        type(dual) :: c
        c%val = a%val / b
        c%der = a%der / b
    end function

    pure elemental function div_rd(a, b) result(c)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        type(dual) :: c
        c%val = a / b%val
        c%der = -a * b%der / (b%val * b%val)
    end function

    !---------------------------------------------------------------------------
    ! Power: d(a^n) = n·a^(n-1)·da
    !---------------------------------------------------------------------------

    pure elemental function pow_di(a, n) result(c)
        type(dual), intent(in) :: a
        integer, intent(in) :: n
        type(dual) :: c
        c%val = a%val ** n
        c%der = real(n, c_double) * (a%val ** (n - 1)) * a%der
    end function

    pure elemental function pow_dr(a, r) result(c)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: r
        type(dual) :: c
        c%val = a%val ** r
        c%der = r * (a%val ** (r - 1.0_c_double)) * a%der
    end function

    !> General power: d(a^b) = a^b · (b·da/a + ln(a)·db)
    pure elemental function pow_dd(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        c%val = a%val ** b%val
        c%der = c%val * (b%val * a%der / a%val + log(a%val) * b%der)
    end function

    !---------------------------------------------------------------------------
    ! Comparison operators (use value only)
    !---------------------------------------------------------------------------

    pure elemental function lt_dd(a, b) result(r)
        type(dual), intent(in) :: a, b
        logical :: r
        r = a%val < b%val
    end function

    pure elemental function lt_dr(a, b) result(r)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        logical :: r
        r = a%val < b
    end function

    pure elemental function lt_rd(a, b) result(r)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        logical :: r
        r = a < b%val
    end function

    pure elemental function gt_dd(a, b) result(r)
        type(dual), intent(in) :: a, b
        logical :: r
        r = a%val > b%val
    end function

    pure elemental function gt_dr(a, b) result(r)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        logical :: r
        r = a%val > b
    end function

    pure elemental function gt_rd(a, b) result(r)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        logical :: r
        r = a > b%val
    end function

    pure elemental function le_dd(a, b) result(r)
        type(dual), intent(in) :: a, b
        logical :: r
        r = a%val <= b%val
    end function

    pure elemental function le_dr(a, b) result(r)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        logical :: r
        r = a%val <= b
    end function

    pure elemental function le_rd(a, b) result(r)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        logical :: r
        r = a <= b%val
    end function

    pure elemental function ge_dd(a, b) result(r)
        type(dual), intent(in) :: a, b
        logical :: r
        r = a%val >= b%val
    end function

    pure elemental function ge_dr(a, b) result(r)
        type(dual), intent(in) :: a
        real(c_double), intent(in) :: b
        logical :: r
        r = a%val >= b
    end function

    pure elemental function ge_rd(a, b) result(r)
        real(c_double), intent(in) :: a
        type(dual), intent(in) :: b
        logical :: r
        r = a >= b%val
    end function

    !---------------------------------------------------------------------------
    ! Mathematical functions
    !---------------------------------------------------------------------------

    !> Square root: d(√a) = da / (2√a)
    pure elemental function sqrt_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = sqrt(a%val)
        c%der = a%der / (2.0_c_double * c%val)
    end function

    !> Exponential: d(e^a) = e^a · da
    pure elemental function exp_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = exp(a%val)
        c%der = c%val * a%der
    end function

    !> Natural logarithm: d(ln a) = da / a
    pure elemental function log_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = log(a%val)
        c%der = a%der / a%val
    end function

    !> Sine: d(sin a) = cos(a) · da
    pure elemental function sin_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = sin(a%val)
        c%der = cos(a%val) * a%der
    end function

    !> Cosine: d(cos a) = -sin(a) · da
    pure elemental function cos_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = cos(a%val)
        c%der = -sin(a%val) * a%der
    end function

    !> Absolute value: d|a| = sign(a) · da
    pure elemental function abs_d(a) result(c)
        type(dual), intent(in) :: a
        type(dual) :: c
        c%val = abs(a%val)
        if (a%val >= 0.0_c_double) then
            c%der = a%der
        else
            c%der = -a%der
        end if
    end function

    !> Maximum of two duals
    pure elemental function max_d(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        if (a%val >= b%val) then
            c = a
        else
            c = b
        end if
    end function

    !> Minimum of two duals
    pure elemental function min_d(a, b) result(c)
        type(dual), intent(in) :: a, b
        type(dual) :: c
        if (a%val <= b%val) then
            c = a
        else
            c = b
        end if
    end function

end module dual_numbers
