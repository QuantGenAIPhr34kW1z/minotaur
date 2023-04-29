module invariants
  use, intrinsic :: iso_c_binding
  implicit none
contains
  pure real(c_double) function mass_residual(bpr, opr)
    real(c_double), intent(in) :: bpr, opr
    ! Synthetic residual: smooth, small in typical corridor, larger near edges.
    mass_residual = abs( (bpr - 0.6_c_double)*1.0e-6_c_double + (opr - 8.0_c_double)*1.0e-7_c_double )
  end function

  pure real(c_double) function energy_residual(t4, t4_max)
    real(c_double), intent(in) :: t4, t4_max
    energy_residual = abs( (t4 / max(t4_max, 1.0_c_double)) - 0.92_c_double ) * 1.0e-9_c_double
  end function
end module
