use std::ffi::CString;
use std::os::unix::io::RawFd;

#[derive(Debug)]
pub enum ChownError {
    BadUser,
    BadGroup,
    IO(std::io::Error),
}

pub fn c(ret: i32) -> Result<i32, std::io::Error> {
    match ret {
	-1 => Err(std::io::Error::last_os_error()),
	_ => Ok(ret)
    }
}

pub fn fchown(fd: RawFd, user: &str, group: &str) -> Result<(), ChownError> {
    let user = CString::new(user).ok().ok_or(ChownError::BadUser)?;
    let group = CString::new(group).ok().ok_or(ChownError::BadGroup)?;
    let user = unsafe {
        libc::getpwnam(user.as_ptr())
            .as_ref()
            .ok_or(ChownError::BadUser)?
    };
    let group = unsafe {
        libc::getgrnam(group.as_ptr())
            .as_ref()
            .ok_or(ChownError::BadGroup)?
    };
    c( unsafe { libc::fchown(fd, user.pw_uid, group.gr_gid) })
        .map(|_| ())
	.map_err(|e| ChownError::IO(e))
    
}

pub fn fchmod(fd: RawFd, mode: u32) -> Result<(), std::io::Error> {
    c(unsafe { libc::fchmod(fd, mode) }).map(|_| ())
}
