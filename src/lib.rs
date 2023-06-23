//! Windows Service execution environment detection.
//!
//! Provides a single function, `is_running_as_windows_service`, that indicates
//! whether the current process is running as a Windows Service or not. This is
//! useful for building binaries that are run both from the command line for
//! development and debugging and under the Windows Service manager.
//!
//! The detection is based on the same approach used in .NET and Golang:
//!
//! * <https://cs.opensource.google/go/x/sys/+/refs/tags/v0.9.0:windows/svc/security.go;l=69>
//! * <https://github.com/dotnet/extensions/blob/f4066026ca06984b07e90e61a6390ac38152ba93/src/Hosting/WindowsServices/src/WindowsServiceHelpers.cs#L26-L31>
#![warn(missing_docs)]
use std::mem::size_of;
use std::os::raw::c_void;
use windows::core::Result;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::Foundation::STATUS_BUFFER_TOO_SMALL;
use windows::Win32::Foundation::STATUS_INFO_LENGTH_MISMATCH;
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::Win32::System::Threading::NtQueryInformationProcess;
use windows::Win32::System::Threading::ProcessBasicInformation;
use windows::Win32::System::Threading::PROCESS_BASIC_INFORMATION;
use windows::Win32::System::WindowsProgramming::NtQuerySystemInformation;
use windows::Win32::System::WindowsProgramming::SystemProcessInformation;
use windows::Win32::System::WindowsProgramming::SYSTEM_PROCESS_INFORMATION;

fn get_current_process_parent_id() -> Result<usize> {
    unsafe {
        let phdl = GetCurrentProcess();
        let mut pinfo = PROCESS_BASIC_INFORMATION::default();
        let mut pinfosz: u32 = 0;
        let res = NtQueryInformationProcess(
            phdl,
            ProcessBasicInformation,
            &mut pinfo as *mut _ as *mut c_void,
            size_of::<PROCESS_BASIC_INFORMATION>() as u32,
            &mut pinfosz,
        );
        CloseHandle(phdl);
        match res {
            Ok(_) => Ok(pinfo.InheritedFromUniqueProcessId),
            Err(e) => Err(e.into()),
        }
    }
}

/// Return whether the current process is running as a Windows Service
pub fn is_running_as_windows_service() -> Result<bool> {
    let ppid = get_current_process_parent_id()?;
    let mut sys_procs_buf: Vec<u8> = Vec::new();
    let mut return_len: u32 = 0;

    // first pass is with zero-size buffer, return_len contains required buffer size
    // second pass actually gets process data
    for _ in 0..2 {
        unsafe {
            match NtQuerySystemInformation(
                SystemProcessInformation,
                (&mut sys_procs_buf).as_mut_ptr() as *mut c_void,
                sys_procs_buf.len() as u32,
                &mut return_len,
            ) {
                Ok(()) => break,
                Err(e) => {
                    if e.code() == STATUS_INFO_LENGTH_MISMATCH.into() {
                        sys_procs_buf.resize(return_len as usize, 0);
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
    }

    if return_len == 0 {
        STATUS_BUFFER_TOO_SMALL.ok()?;
    }

    unsafe {
        let mut ptr = sys_procs_buf.as_mut_ptr();
        let mut count = 0;
        while ptr < sys_procs_buf.as_mut_ptr().add(sys_procs_buf.len()) && count < 1024 {
            let proc = (ptr as *mut SYSTEM_PROCESS_INFORMATION).as_ref().unwrap();
            if proc.UniqueProcessId.0 as usize == ppid {
                let proc_name = if proc.ImageName.Buffer.is_null() {
                    "<null>".to_owned()
                } else {
                    proc.ImageName.Buffer.to_string().unwrap()
                };
                return Ok(proc.SessionId == 0 && proc_name == "services.exe");
            }
            let next_offset = proc.NextEntryOffset as usize;
            if next_offset == 0 {
                break;
            }
            ptr = ptr.add(next_offset);
            count += 1;
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_running_as_windows_service_nonservice() {
        assert!(!is_running_as_windows_service().expect("error during detection"));
    }
}
