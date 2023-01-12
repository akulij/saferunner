mod safemode;
mod tools;

use winapi::shared::ntdef::{HANDLE, LUID};
use winapi::um::winnt::{
    SE_PRIVILEGE_ENABLED, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use winapi::um::{
    processthreadsapi::GetCurrentProcess,
    processthreadsapi::OpenProcessToken,
    securitybaseapi::AdjustTokenPrivileges,
    winbase::LookupPrivilegeValueA,
    winuser::ExitWindowsEx,
    winuser::{EWX_FORCEIFHUNG, EWX_REBOOT},
    lmaccess::NetUserSetInfo,
    lmaccess::USER_INFO_1003,
};

use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::enums::KEY_SET_VALUE;
use winreg::RegKey;

use crate::safemode::{set_safemode, unset_safemode};
use crate::tools::cstr;
use crate::tools::nullptr;
use crate::tools::random_code;
use crate::tools::encode_str;
use crate::tools::cast_ptr;

pub struct User {
    login: String,
    password: String,
}

#[cfg(windows)]
#[doc = "If code executed successfully, code after this function will not be executed"]
pub fn runprog_safemode(user: Option<&User>, program_path: &str, change_user_pass: bool) {
    if let Some(user) = user {
        if change_user_pass {
            if !set_user_password(user) {
                return;
            }
        }
        if !set_autologin(user) {
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
#[doc = "If code executed successfully, code after this function will not be executed"]
pub fn back_to_normalmode() {
    if !unset_safemode() {
        return;
    }
    reboot_pc();
}

fn reboot_pc() {
    // get current process token and adjust privilegies to allow system shutdown
    unsafe {
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

#[doc = "Push data by value to Winlogon registry"]
fn set_logon_data(value: &str, data: &str) -> bool {
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

#[doc = "Add program path to RunOnce registry with support of start in safe mode"]
fn set_runonce_program(path: &str) -> bool {
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

fn set_user_password(user: &User) -> bool {
    let mut buff = encode_str(&user.password.as_str());
    let mut data: USER_INFO_1003 = unsafe { std::mem::zeroed() };
    data.usri1003_password = buff.as_mut_ptr();

    unsafe {
        let status = NetUserSetInfo(nullptr(), encode_str(&user.login).as_mut_ptr(), 1003, cast_ptr(&mut data), nullptr());
        status == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_safeboot() {
        set_safemode();
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
    fn test_password_setter() {
        assert!(set_user_password(&User {login: String::from("test"), password: String::from("testt")}))
    }
}
