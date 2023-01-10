#[allow(temporary_cstring_as_ptr)]
use crate::tools::nullptr;
use crate::tools::cstr;
use crate::tools::random_code;

mod tools;

pub struct User {
    login: String,
    password: String,
}

#[cfg(windows)]
pub fn runprog_safemode(user: Option<&User>, program_path: &str) {
    if let Some(user) = user {
        if set_autologin(user) {
            return;
        }
    }

    if !set_runonce_program(program_path) {
        return;
    }

    if !set_safemode() {
        return;
    }
    reboot_pc();
}

#[cfg(windows)]
pub fn back_to_normalmode() {
    if !unset_safemode() {
        return;
    }
    reboot_pc();
}

fn set_safemode() -> bool {
    run_bcd("/set {current} safeboot network")
}

fn unset_safemode() -> bool {
    run_bcd("/deletevalue {current} safeboot")
}

fn run_bcd(command: &str) -> bool {
    use winapi::ctypes::c_void;
    use winapi::um::shellapi::ShellExecuteA;
    use winapi::um::winnt::PVOID;
    use winapi::um::winuser::SW_SHOWNORMAL;
    use winapi::um::wow64apiset::Wow64DisableWow64FsRedirection;
    use winapi::um::wow64apiset::Wow64RevertWow64FsRedirection;

    let is_wow_enabled = tools::is_wow64();
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

fn reboot_pc() {
    use winapi::um::winuser::ExitWindowsEx;
    use winapi::um::winuser::{EWX_FORCEIFHUNG, EWX_REBOOT};

    unsafe {
        use winapi::shared::ntdef::{HANDLE, LUID};
        use winapi::um::winnt::SE_PRIVILEGE_ENABLED;
        use winapi::um::winnt::TOKEN_ADJUST_PRIVILEGES;
        use winapi::um::winnt::TOKEN_PRIVILEGES;
        use winapi::um::winnt::TOKEN_QUERY;

        use winapi::um::{
            processthreadsapi::GetCurrentProcess, processthreadsapi::OpenProcessToken,
            securitybaseapi::AdjustTokenPrivileges, winbase::LookupPrivilegeValueA,
        };

        let mut token: HANDLE = std::mem::zeroed();
        let mut luid: LUID = std::mem::zeroed();

        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        );
        LookupPrivilegeValueA(nullptr(), cstr("SeShutdownPrivilege").as_ptr(), &mut luid);

        let mut tp: TOKEN_PRIVILEGES = std::mem::zeroed();
        tp.PrivilegeCount = 1;
        tp.Privileges[0].Luid = luid;
        tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        AdjustTokenPrivileges(token, 0, &mut tp, 0, nullptr(), nullptr());
    }

    unsafe {
        ExitWindowsEx(EWX_REBOOT, EWX_FORCEIFHUNG);
        use winapi::um::errhandlingapi::GetLastError;
        println!("REB: {}", GetLastError());
    }
}

fn set_autologin(user: &User) -> bool {
    if !set_logon_data("DefaultUserName", user.login.as_str()) {
        return false;
    }
    if !set_logon_data("DefaultPassword", user.password.as_str()) {
        return false;
    }
    if !set_logon_data("AutoAdminLogon", "1") {
        return false;
    }

    true
}

fn set_logon_data(value: &str, data: &str) -> bool {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::enums::KEY_SET_VALUE;
    use winreg::RegKey;

    let regkey = match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
        KEY_SET_VALUE,
    ) {
        Ok(v) => v,
        Err(_) => return false,
    };

    regkey.set_value(value, &data).unwrap();

    true
}

fn set_runonce_program(path: &str) -> bool {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::enums::KEY_SET_VALUE;
    use winreg::RegKey;

    let regkey = match RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\RunOnce",
        KEY_SET_VALUE,
    ) {
        Ok(v) => v,
        Err(_) => return false,
    };

    regkey
        .set_value(random_code(6).as_str(), &(String::from("*") + path))
        .unwrap();

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_safeboot() {
        // set_safemode();
    }

    #[test]
    fn test_unset_safeboot() {
        unset_safemode();
    }

    #[test]
    fn test_autologin() {
        assert!(set_autologin(&User {
            login: "test".to_string(),
            password: "test".to_string()
        }));
    }
    #[test]
    fn test_runonce() {
        assert!(set_runonce_program(
            "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe"
        ));
    }

    #[test]
    fn test_reboot() {
        // reboot_pc();
    }
}
