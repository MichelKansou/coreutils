use std::{
    convert::From,
    env::{self, VarError},
    io::Error as IoError,
    mem,
    path::PathBuf,
};

use libc::stat;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Var(VarError),
    Io(IoError),
}

impl From<VarError> for Error {
    fn from(err: VarError) -> Error {
        Error::Var(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

pub fn current_dir_logical() -> Result<PathBuf> {
    let pwd = env::var("PWD")?;

    let (mut logical, mut physical): (stat, stat) =
        unsafe { (mem::uninitialized(), mem::uninitialized()) };

    let (mut stat1, mut stat2): (bool, bool) = unsafe { (mem::uninitialized(), mem::uninitialized()) };

    // Validity check
    // if we can get both fisical and logical paths stat, check they are the same inode
    if pwd.starts_with('/') {
        unsafe {
            stat1 = dbg!(stat(pwd.as_ptr() as *const i8, &mut logical) == 0);
            stat2 = dbg!(stat(".".as_ptr() as *const i8, &mut physical) == 0);
        }

        if stat1 && stat2 && logical.st_ino == physical.st_ino {
            return Ok(PathBuf::from(pwd));
        }
    }
    Err(Error::Io(IoError::last_os_error()))
}