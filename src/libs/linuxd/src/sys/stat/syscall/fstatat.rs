// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use crate::{
    message::{
        LinuxDaemonLongMessage,
        LinuxDaemonMessagePart,
        MessagePartitioner,
    },
    sys::stat::{
        message::{
            FileStatRequest,
            FileStatResponse,
        },
        stat,
    },
    LinuxDaemonMessage,
    LinuxDaemonMessageHeader,
};
use ::alloc::{
    string::ToString,
    vec::Vec,
};
use ::nvx::{
    ipc::Message,
    pm::ProcessIdentifier,
    sys::error::ErrorCode,
};

//==================================================================================================
// Standalone Functions
//==================================================================================================

///
/// # Description
///
/// The `fstatat()` system call obtains information about a file.
///
/// # Parameters
///
/// - `dirfd`: Directory file descriptor.
/// - `path`: Path to the file.
/// - `buf`: Buffer to store file information.
///
/// # Returns
///
/// Upon successful completion, `0` is returned. Upon failure, a negative error code is returned
/// instead.
///
pub fn fstatat(dirfd: i32, path: &str, buf: &mut stat, flag: i32) -> i32 {
    // Send request.
    let status: i32 = fstatat_request(dirfd, path, flag);
    if status != 0 {
        return status;
    }

    // Wait for response.
    fstatat_response(buf)
}

///
/// # Description
///
/// This function sends a request to the daemon to execute the `fstatat()` system call.
///
/// # Parameters
///
/// - `dirfd`: Directory file descriptor.
/// - `path`: Path to the file.
/// - `flag`: Flags.
///
/// # Returns
///
/// Upon successful completion, `0` is returned. Upon failure, a negative error code is returned
/// instead.
///
fn fstatat_request(dirfd: i32, path: &str, flag: i32) -> i32 {
    let pid: ProcessIdentifier = match ::nvx::pm::getpid() {
        Ok(pid) => pid,
        Err(e) => return e.code.into_errno(),
    };

    let request: FileStatRequest = FileStatRequest::new(dirfd, path.to_string(), flag);
    let requests: Vec<Message> = match request.into_parts(pid) {
        Ok(requests) => requests,
        Err(e) => return e.code.into_errno(),
    };

    for request in requests {
        match ::nvx::ipc::send(&request) {
            Ok(_) => (),
            Err(e) => return e.code.into_errno(),
        }
    }

    0
}

///
/// # Description
///
/// This function waits for the response of the `fstatat()` system call.
///
/// # Parameters
///
/// - `buf`: Buffer to store file information.
///
/// # Returns
///
/// Upon successful completion, `0` is returned. Upon failure, a negative error code is returned
/// instead.
///
fn fstatat_response(buf: &mut stat) -> i32 {
    let capacity: usize = stat::SIZE.div_ceil(LinuxDaemonMessagePart::PAYLOAD_SIZE);

    let mut assembler: LinuxDaemonLongMessage = match LinuxDaemonLongMessage::new(capacity) {
        Ok(assembler) => assembler,
        Err(e) => return e.code.into_errno(),
    };

    loop {
        let response: Message = match ::nvx::ipc::recv() {
            Ok(response) => response,
            Err(e) => break e.code.into_errno(),
        };

        // Check whether system call succeeded or not.
        if response.status != 0 {
            // System call failed, parse error code and return it.
            match ErrorCode::try_from(response.status) {
                Ok(e) => break e.into_errno(),
                Err(_) => break ErrorCode::InvalidMessage.into_errno(),
            }
        } else {
            // System call succeeded, parse response.
            match LinuxDaemonMessage::try_from_bytes(response.payload) {
                Ok(message) => match message.header {
                    LinuxDaemonMessageHeader::FileStatResponsePart => {
                        let part: LinuxDaemonMessagePart =
                            LinuxDaemonMessagePart::from_bytes(message.payload);

                        if let Err(e) = assembler.add_part(part) {
                            break e.code.into_errno();
                        }

                        if !assembler.is_complete() {
                            continue;
                        }

                        let parts: Vec<LinuxDaemonMessagePart> = assembler.take_parts();

                        match FileStatResponse::from_parts(&parts) {
                            Ok(response) => {
                                *buf = response.stat;
                                break 0;
                            },
                            Err(_) => break ErrorCode::InvalidMessage.into_errno(),
                        }
                    },
                    _ => break ErrorCode::InvalidMessage.into_errno(),
                },
                Err(_) => break ErrorCode::InvalidMessage.into_errno(),
            }
        }
    }
}