module thermo
  use, intrinsic :: iso_c_binding
  implicit none
contains
  pure real(c_double) function clamp(x, lo, hi)
    real(c_double), intent(in) :: x, lo, hi
    clamp = min(max(x, lo), hi)
  end function

  ! Synthetic atmosphere proxy: returns a smooth multiplier with altitude and mach.
  pure real(c_double) function regime_factor(mach, alt_km)
    real(c_double), intent(in) :: mach, alt_km
    real(c_double) :: a, m
    a = clamp(alt_km, 0.0_c_double, 20.0_c_double)
    m = clamp(mach, 0.0_c_double, 0.95_c_double)
    regime_factor = (1.0_c_double - 0.02_c_double*a) * (1.0_c_double + 0.15_c_double*m)
  end function
end module
