! RSPhysics I/O adapter; original scientific sources remain in upstream/.
module rsp_io
  use iso_c_binding
  implicit none
  character(len=4096) :: rsp_directory=''
  character(len=512) :: rsp_message=''
  integer :: rsp_ios=0
  logical :: rsp_failed=.false.
  logical(c_bool),bind(C) :: rsp_mie_used=.false.
contains
  function rsp_path(name) result(path)
    character(len=*), intent(in) :: name
    character(len=4096) :: path
    path=trim(rsp_directory)//'/'//trim(name)
    if(index(name,'..')>0.or.index(name,':')>0.or.name(1:1)=='/'.or.name(1:1)==achar(92)) then
      rsp_failed=.true.
      rsp_message='Auxiliary filenames must be relative to the calculation directory'
      path=trim(rsp_directory)//'/invalid-auxiliary-path'
    endif
  end function
end module
