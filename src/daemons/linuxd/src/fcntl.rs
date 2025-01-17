// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use ::alloc::ffi::CString;
use ::core::{
    ffi,
    str,
};
use ::linuxd::{
    fcntl,
    fcntl::message::{
        FileAdvisoryInformationRequest,
        FileAdvisoryInformationResponse,
        FileSpaceControlRequest,
        FileSpaceControlResponse,
        OpenAtRequest,
        OpenAtResponse,
        ReadLinkAtRequest,
        ReadLinkAtResponse,
        RenameAtRequest,
        RenameAtResponse,
        SymbolicLinkAtRequest,
        SymbolicLinkAtResponse,
        UnlinkAtRequest,
        UnlinkAtResponse,
    },
    message::MessagePartitioner,
    sys::{
        stat::{
            message::{
                FileStatAtRequest,
                FileStatAtResponse,
                FileStatRequest,
            },
            stat,
        },
        types::{
            mode_t,
            off_t,
        },
    },
    time::timespec,
};
use ::nvx::{
    ipc::Message,
    pm::ProcessIdentifier,
    sys::error::{
        Error,
        ErrorCode,
    },
};

//==================================================================================================
// do_openat
//==================================================================================================

pub fn do_open_at(pid: ProcessIdentifier, request: OpenAtRequest) -> Message {
    trace!("openat(): pid={:?}, request={:?}", pid, request);

    let dirfd: i32 = request.dirfd;
    let flags: ffi::c_int = request.flags;
    let mode: mode_t = request.mode;

    let pathname: &str = match str::from_utf8(&request.pathname) {
        Ok(pathname) => pathname,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    let dirfd: LibcAtFlags = LibcAtFlags::from(dirfd);
    let flags: LibcFileFlags = match LibcFileFlags::try_from(flags) {
        Ok(flags) => flags,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };
    let mode: LibcFileMode = match LibcFileMode::try_from(mode) {
        Ok(mode) => mode,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    debug!(
        "libc::openat(): dirfd={:?}, pathname={:?}, flags={:?}, mode={:?}",
        dirfd.inner(),
        pathname,
        flags.inner(),
        mode.inner()
    );
    match unsafe {
        libc::openat(
            dirfd.inner(),
            pathname.as_bytes().as_ptr() as *const i8,
            flags.inner(),
            mode.inner(),
        )
    } {
        fd if fd >= 0 => {
            debug!("libc::openat(): fd={:?}", fd);
            OpenAtResponse::build(pid, fd)
        },
        errno => {
            debug!("libc::openat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(errno).expect("unknown error code {error}");
            crate::build_error(pid, error)
        },
    }
}

//==================================================================================================
// do_unlink_at
//==================================================================================================

pub fn do_unlink_at(pid: ProcessIdentifier, request: UnlinkAtRequest) -> Message {
    trace!("unlinkat(): pid={:?}, request={:?}", pid, request);

    let dirfd: i32 = request.dirfd;
    let flags: ffi::c_int = request.flags;

    let pathname: &str = match str::from_utf8(&request.pathname) {
        Ok(pathname) => pathname,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    let dirfd: LibcAtFlags = LibcAtFlags::from(dirfd);
    let flags: LibcFileFlags = match LibcFileFlags::try_from(flags) {
        Ok(flags) => flags,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    debug!(
        "libc::unlinkat(): dirfd={:?}, pathname={:?}, flags={:?}",
        dirfd.inner(),
        pathname,
        flags.inner()
    );
    match unsafe {
        libc::unlinkat(dirfd.inner(), pathname.as_bytes().as_ptr() as *const i8, flags.inner())
    } {
        ret if ret == 0 => {
            debug!("libc::unlinkat(): success");
            UnlinkAtResponse::build(pid, ret)
        },
        errno => {
            debug!("libc::unlinkat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(errno).expect("unknown error code {error}");
            crate::build_error(pid, error)
        },
    }
}

//==================================================================================================
// do_rename_at
//==================================================================================================

pub fn do_rename_at(pid: ProcessIdentifier, request: RenameAtRequest) -> Message {
    trace!("renameat(): pid={:?}, request={:?}", pid, request);

    let olddirfd: i32 = request.olddirfd;
    let newdirfd: i32 = request.newdirfd;

    let oldpath: &str = match str::from_utf8(&request.oldpath) {
        Ok(oldpath) => oldpath,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    let newpath: &str = match str::from_utf8(&request.newpath) {
        Ok(newpath) => newpath,
        Err(_) => return crate::build_error(pid, ErrorCode::InvalidMessage),
    };

    let olddirfd: LibcAtFlags = LibcAtFlags::from(olddirfd);
    let newdirfd: LibcAtFlags = LibcAtFlags::from(newdirfd);

    debug!(
        "libc::renameat(): olddirfd={:?}, oldpath={:?}, newdirfd={:?}, newpath={:?}",
        olddirfd.inner(),
        oldpath,
        newdirfd.inner(),
        newpath
    );
    match unsafe {
        libc::renameat(
            olddirfd.inner(),
            oldpath.as_bytes().as_ptr() as *const i8,
            newdirfd.inner(),
            newpath.as_bytes().as_ptr() as *const i8,
        )
    } {
        ret if ret == 0 => {
            debug!("libc::renameat(): success");
            RenameAtResponse::build(pid, ret)
        },
        errno => {
            debug!("libc::renameat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(errno).expect("unknown error code {error}");
            crate::build_error(pid, error)
        },
    }
}

//==================================================================================================
// do_fstatat
//==================================================================================================

pub fn do_fstat_at(pid: ProcessIdentifier, request: FileStatAtRequest) -> Vec<Message> {
    trace!("fstatat(): pid={:?}, request={:?}", pid, request);

    let dirfd: i32 = request.dirfd;
    let dirfd: LibcAtFlags = LibcAtFlags::from(dirfd);
    let flag: i32 = request.flag;
    let flag: LibcFileFlags = LibcFileFlags(flag);
    let path: CString = match CString::new(request.path.as_str()) {
        Ok(c_string) => c_string,
        Err(_) => return vec![crate::build_error(pid, ErrorCode::InvalidMessage)],
    };

    let mut st: libc::stat = unsafe { core::mem::zeroed() };

    debug!("libc::fstatat(): dirfd={:?}, path={:?}, flag={:?}", dirfd.inner(), path, flag.inner());
    match unsafe {
        libc::fstatat(dirfd.inner(), path.as_ptr(), &mut st as *mut libc::stat, flag.inner())
    } {
        0 => {
            debug!("libc::fstatat(): success");

            let stat = stat {
                st_dev: st.st_dev,
                st_ino: st.st_ino,
                st_mode: st.st_mode,
                st_nlink: st.st_nlink,
                st_uid: st.st_uid,
                st_gid: st.st_gid,
                st_rdev: st.st_rdev,
                st_size: st.st_size,
                st_atim: timespec {
                    tv_sec: st.st_atime,
                    tv_nsec: st.st_atime_nsec,
                },
                st_mtim: timespec {
                    tv_sec: st.st_mtime,
                    tv_nsec: st.st_mtime_nsec,
                },
                st_ctim: timespec {
                    tv_sec: st.st_ctime,
                    tv_nsec: st.st_ctime_nsec,
                },
                st_blksize: st.st_blksize,
                st_blocks: st.st_blocks,
            };

            // Print size of stat structure.
            debug!("libc::fstatat(): size of stat={:?}", core::mem::size_of::<stat>());
            let response = FileStatAtResponse::new(stat);

            match response.into_parts(pid) {
                Ok(messages) => messages,
                Err(e) => vec![crate::build_error(pid, e.code)],
            }
        },
        _ => {
            let errno: i32 = unsafe { *libc::__errno_location() };
            debug!("libc::fstatat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            vec![crate::build_error(pid, error)]
        },
    }
}

//==================================================================================================
// do_posix_fallocate
//==================================================================================================

pub fn do_posix_fallocate(pid: ProcessIdentifier, request: FileSpaceControlRequest) -> Message {
    trace!("posix_fallocate(): pid={:?}, request={:?}", pid, request);

    let fd: i32 = request.fd;
    let offset: off_t = request.offset;
    let len: off_t = request.len;

    debug!("libc::posix_fallocate(): fd={:?}, offset={:?}, len={:?}", fd, offset, len);
    match unsafe { libc::posix_fallocate(fd, offset, len) } {
        0 => {
            debug!("libc::posix_fallocate(): success");
            FileSpaceControlResponse::build(pid, 0)
        },
        errno => {
            debug!("libc::posix_fallocate(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            crate::build_error(pid, error)
        },
    }
}

//==================================================================================================
// do_posix_fadvise
//==================================================================================================

pub fn do_posix_fadvise(
    pid: ProcessIdentifier,
    request: FileAdvisoryInformationRequest,
) -> Message {
    trace!("posix_fadvise(): pid={:?}, request={:?}", pid, request);

    let fd: i32 = request.fd;
    let offset: off_t = request.offset;
    let len: off_t = request.len;
    let advice: LibcFileAdvice = match LibcFileAdvice::try_from(request.advice) {
        Ok(advice) => advice,
        Err(e) => return crate::build_error(pid, e.code),
    };

    debug!(
        "libc::posix_fadvise(): fd={:?}, offset={:?}, len={:?}, advice={:?}",
        fd,
        offset,
        len,
        advice.inner()
    );
    match unsafe { libc::posix_fadvise(fd, offset, len, advice.inner()) } {
        0 => {
            debug!("libc::posix_fadvise(): success");
            FileAdvisoryInformationResponse::build(pid, 0)
        },
        errno => {
            debug!("libc::posix_fadvise(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            crate::build_error(pid, error)
        },
    }
}

//==================================================================================================
// do_fstat()
//==================================================================================================

pub fn do_fstat(pid: ProcessIdentifier, request: FileStatRequest) -> Vec<Message> {
    trace!("fstatat(): pid={:?}, request={:?}", pid, request);

    let fd: i32 = request.fd;

    let mut st: libc::stat = unsafe { core::mem::zeroed() };

    debug!("libc::fstat(): fd={:?}", fd);
    match unsafe { libc::fstat(fd, &mut st) } {
        0 => {
            debug!("libc::fstatat(): success");

            let stat = stat {
                st_dev: st.st_dev,
                st_ino: st.st_ino,
                st_mode: st.st_mode,
                st_nlink: st.st_nlink,
                st_uid: st.st_uid,
                st_gid: st.st_gid,
                st_rdev: st.st_rdev,
                st_size: st.st_size,
                st_atim: timespec {
                    tv_sec: st.st_atime,
                    tv_nsec: st.st_atime_nsec,
                },
                st_mtim: timespec {
                    tv_sec: st.st_mtime,
                    tv_nsec: st.st_mtime_nsec,
                },
                st_ctim: timespec {
                    tv_sec: st.st_ctime,
                    tv_nsec: st.st_ctime_nsec,
                },
                st_blksize: st.st_blksize,
                st_blocks: st.st_blocks,
            };

            // Print size of stat structure.
            debug!("libc::fstatat(): size of stat={:?}", core::mem::size_of::<stat>());
            let response = FileStatAtResponse::new(stat);

            match response.into_parts(pid) {
                Ok(messages) => messages,
                Err(e) => vec![crate::build_error(pid, e.code)],
            }
        },
        _ => {
            let errno: i32 = unsafe { *libc::__errno_location() };
            debug!("libc::fstatat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            vec![crate::build_error(pid, error)]
        },
    }
}

//==================================================================================================
// do_symlinkat()
//==================================================================================================

pub fn do_symlinkat(pid: ProcessIdentifier, request: SymbolicLinkAtRequest) -> Vec<Message> {
    trace!("symlinkat(): pid={:?}, request={:?}", pid, request);

    let target: CString = match CString::new(request.target.as_str()) {
        Ok(target) => target,
        Err(_) => return vec![crate::build_error(pid, ErrorCode::InvalidMessage)],
    };

    let newdirfd: i32 = request.dirfd;
    let newdirfd: LibcAtFlags = LibcAtFlags::from(newdirfd);

    let linkpath: CString = match CString::new(request.linkpath.as_str()) {
        Ok(linkpath) => linkpath,
        Err(_) => return vec![crate::build_error(pid, ErrorCode::InvalidMessage)],
    };

    debug!(
        "libc::symlinkat(): oldpath={:?}, newdirfd={:?}, newpath={:?}",
        target,
        newdirfd.inner(),
        linkpath
    );
    match unsafe { libc::symlinkat(target.as_ptr(), newdirfd.inner(), linkpath.as_ptr()) } {
        0 => {
            debug!("libc::symlinkat(): success");
            vec![SymbolicLinkAtResponse::build(pid, 0)]
        },
        _ => {
            let errno: i32 = unsafe { *libc::__errno_location() };
            debug!("libc::symlinkat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            vec![crate::build_error(pid, error)]
        },
    }
}

//==================================================================================================
// do_readlinkat()
//==================================================================================================

pub fn do_readlinkat(pid: ProcessIdentifier, request: ReadLinkAtRequest) -> Vec<Message> {
    trace!("readlinkat(): pid={:?}, request={:?}", pid, request);

    let dirfd: i32 = request.dirfd;
    let dirfd: LibcAtFlags = LibcAtFlags::from(dirfd);

    let path: CString = match CString::new(request.path.as_str()) {
        Ok(path) => path,
        Err(_) => return vec![crate::build_error(pid, ErrorCode::InvalidMessage)],
    };

    // TODO: Have a system-wide constant for this.
    let mut buf: Vec<u8> = vec![0u8; ReadLinkAtResponse::BUFFER_SIZE_MAX];

    debug!(
        "libc::readlinkat(): dirfd={:?}, path={:?}, capacity={:?}",
        dirfd.inner(),
        path,
        buf.capacity()
    );
    match unsafe {
        libc::readlinkat(dirfd.inner(), path.as_ptr(), buf.as_mut_ptr() as *mut i8, buf.capacity())
    } {
        len if len >= 0 => {
            debug!("libc::readlinkat(): success");

            buf.truncate(len as usize);

            let response: ReadLinkAtResponse = match ReadLinkAtResponse::new(buf) {
                Ok(response) => response,
                Err(e) => return vec![crate::build_error(pid, e.code)],
            };

            match response.into_parts(pid) {
                Ok(messages) => messages,
                Err(e) => vec![crate::build_error(pid, e.code)],
            }
        },
        _ => {
            let errno: i32 = unsafe { *libc::__errno_location() };
            debug!("libc::readlinkat(): errno={:?}", errno);
            let error: ErrorCode = ErrorCode::try_from(-errno)
                .unwrap_or_else(|_| panic!("unknown error code {errno}"));
            vec![crate::build_error(pid, error)]
        },
    }
}

//==================================================================================================

struct LibcFileFlags(libc::c_int);

impl LibcFileFlags {
    fn inner(&self) -> libc::c_int {
        self.0
    }

    fn try_from(flags: ffi::c_int) -> Result<LibcFileFlags, Error> {
        let flag_mappings: [(ffi::c_int, i32); 7] = [
            (fcntl::O_APPEND, libc::O_APPEND),
            (fcntl::O_CREAT, libc::O_CREAT),
            (fcntl::O_EXCL, libc::O_EXCL),
            (fcntl::O_RDONLY, libc::O_RDONLY),
            (fcntl::O_RDWR, libc::O_RDWR),
            (fcntl::O_TRUNC, libc::O_TRUNC),
            (fcntl::O_WRONLY, libc::O_WRONLY),
        ];

        // TODO: check for unsupported flags.

        let mut libc_flags: libc::c_int = 0;
        for (nanvix_flag, f) in flag_mappings.iter() {
            if (flags & nanvix_flag) == *nanvix_flag {
                libc_flags |= *f;
            }
        }

        Ok(LibcFileFlags(libc_flags))
    }
}

struct LibcFileMode(libc::mode_t);

impl LibcFileMode {
    fn inner(&self) -> libc::mode_t {
        self.0
    }

    fn try_from(mode: mode_t) -> Result<LibcFileMode, Error> {
        let mode_mappings: [(mode_t, u32); 12] = [
            (fcntl::S_IRWXU, libc::S_IRWXU),
            (fcntl::S_IRUSR, libc::S_IRUSR),
            (fcntl::S_IWUSR, libc::S_IWUSR),
            (fcntl::S_IXUSR, libc::S_IXUSR),
            (fcntl::S_IRWXG, libc::S_IRWXG),
            (fcntl::S_IRGRP, libc::S_IRGRP),
            (fcntl::S_IWGRP, libc::S_IWGRP),
            (fcntl::S_IXGRP, libc::S_IXGRP),
            (fcntl::S_IRWXO, libc::S_IRWXO),
            (fcntl::S_IROTH, libc::S_IROTH),
            (fcntl::S_IWOTH, libc::S_IWOTH),
            (fcntl::S_IXOTH, libc::S_IXOTH),
        ];

        // TODO: check for unsupported flags.

        let mut libc_mode: libc::mode_t = 0;
        for (nanvix_mode, m) in mode_mappings.iter() {
            if (mode & nanvix_mode) == *nanvix_mode {
                libc_mode |= *m;
            }
        }

        Ok(LibcFileMode(libc_mode))
    }
}

struct LibcAtFlags(libc::c_int);

impl LibcAtFlags {
    fn inner(&self) -> libc::c_int {
        self.0
    }

    fn from(flags: ffi::c_int) -> LibcAtFlags {
        let libc_flags: libc::c_int = match flags {
            fcntl::AT_FDCWD => libc::AT_FDCWD,
            fcntl::AT_REMOVEDIR => libc::AT_REMOVEDIR,
            _ => flags,
        };

        LibcAtFlags(libc_flags)
    }
}

pub struct LibcFileAdvice(libc::c_int);

impl LibcFileAdvice {
    fn inner(&self) -> libc::c_int {
        self.0
    }

    fn try_from(advice: i32) -> Result<LibcFileAdvice, Error> {
        let libc_advice: libc::c_int = match advice {
            fcntl::POSIX_FADV_NORMAL => libc::POSIX_FADV_NORMAL,
            fcntl::POSIX_FADV_RANDOM => libc::POSIX_FADV_RANDOM,
            fcntl::POSIX_FADV_SEQUENTIAL => libc::POSIX_FADV_SEQUENTIAL,
            fcntl::POSIX_FADV_WILLNEED => libc::POSIX_FADV_WILLNEED,
            fcntl::POSIX_FADV_DONTNEED => libc::POSIX_FADV_DONTNEED,
            fcntl::POSIX_FADV_NOREUSE => libc::POSIX_FADV_NOREUSE,
            _ => return Err(Error::new(ErrorCode::InvalidArgument, "invalid advice")),
        };

        Ok(LibcFileAdvice(libc_advice))
    }
}
