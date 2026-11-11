! Direct entry point. Status: 0 success; 1 model/input error; 2 I/O error.
integer(c_int) function rsp_fortran_run(directory) bind(C) result(status)
  use iso_c_binding
  use rsp_io
  implicit none
  character(kind=c_char), intent(in) :: directory(*)
  integer :: i,ios,unit
  logical :: ier
  integer :: iwr
  common /sixs_ier/ iwr,ier
  status=2
  rsp_directory=''
  do i=1,len(rsp_directory)-1
    if(directory(i)==c_null_char) exit
    rsp_directory(i:i)=directory(i)
  enddo
  rsp_failed=.false.
  rsp_message=''
  ier=.false.
  open(unit=45,file=rsp_path('input.cards'),status='old',action='read',iostat=ios,iomsg=rsp_message)
  if(ios/=0) return
  open(unit=46,file=rsp_path('report.txt'),status='replace',action='write',iostat=ios,iomsg=rsp_message)
  if(ios/=0) then
    close(45)
    return
  endif
  call rsp_main()
  status=0
  if(ier.or.rsp_failed) status=1
  ! CLOSE on an unconnected unit is a permitted no-op; avoid INQUIRE's
  ! compiler/runtime logical ABI difference in the pinned wasm32 toolchain.
  close(10,iostat=ios)
  close(45,iostat=ios)
  close(46,iostat=ios)
  if(status/=0) then
    open(newunit=unit,file=rsp_path('error.txt'),status='replace',iostat=ios)
    if(ios==0) then
      write(unit,'(a)') trim(rsp_message)
      close(unit)
    endif
  endif
end function
