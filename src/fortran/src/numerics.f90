module numerics
  use, intrinsic :: iso_c_binding
  implicit none
contains
  pure real(c_double) function norm2(a, b, c)
    real(c_double), intent(in) :: a, b, c
    norm2 = sqrt(a*a + b*b + c*c)
  end function
end module
