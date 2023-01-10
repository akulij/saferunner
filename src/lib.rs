mod tools;

pub struct User {
    login: String,
    pasword: String,
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
    run_bcd("/set safeboot network")
}

fn unset_safemode() -> bool {
    run_bcd("/deletevalue safeboot")
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

    use tools::cstr;
    use tools::nullptr;

    unsafe {
        ShellExecuteA(
            nullptr(),
            cstr("runas"),
            cstr("BCDEdit.exe"),
            cstr(command),
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
        ExitWindowsEx(EWX_REBOOT, EWX_FORCEIFHUNG);
    }
}

fn set_autologin(user: &User) -> bool {
    set_logon_data("DefaultUserName", user.login.as_str());
    set_logon_data("DefaultPassword", user.pasword.as_str());
    set_logon_data("AutoAdminLogon", "1");

    true
}

fn set_logon_data(value: &str, data: &str) -> bool {
    use tools::cast_pointer;
    use tools::encode_str;
    use winapi::um::winnt::REG_EXPAND_SZ;
    use winapi::um::winreg::RegSetKeyValueW;
    use winapi::um::winreg::HKEY_LOCAL_MACHINE;

    unsafe {
        RegSetKeyValueW(
            HKEY_LOCAL_MACHINE,
            encode_str("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon"),
            encode_str(value),
            REG_EXPAND_SZ,
            cast_pointer(encode_str(data)),
            data.len() as u32 + 1,
        );
    }

    true
}

fn set_runonce_program(path: &str) -> bool {
    use tools::cast_pointer;
    use tools::encode_str;
    use tools::random_code;
    use winapi::um::winnt::REG_EXPAND_SZ;
    use winapi::um::winreg::RegSetKeyValueW;
    use winapi::um::winreg::HKEY_LOCAL_MACHINE;

    unsafe {
        RegSetKeyValueW(
            HKEY_LOCAL_MACHINE,
            encode_str("Software\\Microsoft\\Windows\\CurrentVersion\\RunOnce"),
            encode_str(random_code(6).as_str()),
            REG_EXPAND_SZ,
            cast_pointer(encode_str(path)),
            path.len() as u32 + 1,
        );
    }

    true
}
