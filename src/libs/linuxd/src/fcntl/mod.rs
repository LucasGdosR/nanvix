// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Configuration
//==================================================================================================

#![allow(non_camel_case_types)]

//==================================================================================================
// Imports
//==================================================================================================

use crate::sys::types::mode_t;
use core::ffi;

//==================================================================================================
// Modules
//==================================================================================================

pub mod message;

cfg_if::cfg_if! {
    if #[cfg(feature = "syscall")] {
        mod syscall;
        pub use self::syscall::{
            openat,
            unlinkat,
            renameat,
            posix_fallocate,
            posix_fadvise,
            symlinkat,
            readlinkat
        };
    }
}

//==================================================================================================

pub const O_APPEND: ffi::c_int = 1 << 0;
pub const O_CREAT: ffi::c_int = 1 << 1;
pub const O_EXCL: ffi::c_int = 1 << 2;
pub const O_TRUNC: ffi::c_int = 1 << 3;
pub const O_RDONLY: ffi::c_int = 1 << 4;
pub const O_WRONLY: ffi::c_int = 1 << 5;
pub const O_RDWR: ffi::c_int = 1 << 6;

pub const S_IRWXU: mode_t = 0o700;
pub const S_IRUSR: mode_t = 0o400;
pub const S_IWUSR: mode_t = 0o200;
pub const S_IXUSR: mode_t = 0o100;
pub const S_IRWXG: mode_t = 0o070;
pub const S_IRGRP: mode_t = 0o040;
pub const S_IWGRP: mode_t = 0o020;
pub const S_IXGRP: mode_t = 0o010;
pub const S_IRWXO: mode_t = 0o007;
pub const S_IROTH: mode_t = 0o004;
pub const S_IWOTH: mode_t = 0o002;
pub const S_IXOTH: mode_t = 0o001;

pub const AT_REMOVEDIR: i32 = 0x200;
pub const AT_FDCWD: i32 = -100;

/// The application has no advice to give on its behavior with respect to the specified data
pub const POSIX_FADV_NORMAL: i32 = 0;
/// The application expects to access the specified data sequentially from lower offsets to higher offsets.
pub const POSIX_FADV_SEQUENTIAL: i32 = 1;
/// The application expects to access the specified data in a random order.
pub const POSIX_FADV_RANDOM: i32 = 2;
/// The specified data will be accessed in the near future.
pub const POSIX_FADV_WILLNEED: i32 = 3;
/// The specified data will not be accessed in the near future.
pub const POSIX_FADV_DONTNEED: i32 = 4;
/// The specified data will be accessed once and then will not be used again.
pub const POSIX_FADV_NOREUSE: i32 = 5;
