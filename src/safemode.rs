use winapi::ctypes::c_void;
use winapi::um::winnt::PVOID;
use winapi::um::{
    shellapi::ShellExecuteA, winuser::SW_SHOWNORMAL, wow64apiset::Wow64DisableWow64FsRedirection,
    wow64apiset::Wow64RevertWow64FsRedirection,
};

use crate::tools::cstr;
use crate::tools::nullptr;

pub fn set_safemode() -> bool {
    run_bcd("/set {current} safeboot network")
}

pub fn unset_safemode() -> bool {
    run_bcd("/deletevalue {current} safeboot")
}

fn run_bcd(command: &str) -> bool {
    let is_wow_enabled = crate::tools::is_wow64();
    let mut revert_holder: PVOID = std::ptr::null::<c_void>().cast_mut();

    if is_wow_enabled {
        unsafe {
            Wow64DisableWow64FsRedirection(&mut revert_holder);
        }
    }

    unsafe {
        ShellExecuteA(
            nullptr(),
            cstr("runas").as_ptr(),
            cstr("BCDEdit.exe").as_ptr(),
            cstr(command).as_ptr(),
            nullptr(),
            SW_SHOWNORMAL,
        );
    }

    if is_wow_enabled {
        unsafe {
            Wow64RevertWow64FsRedirection(revert_holder);
        }
    }

    true
}
