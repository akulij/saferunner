pub struct User {
    login: String,
    pasword: String,
}

pub fn runprog_safemode(user: Option<&User>, program: &str) {
    if let Some(user) = user {
        set_autologin(user);
    }

    set_runonce_program(program);

    set_safemode();
    reboot_pc();
}

pub fn back_to_normalmode() {
}
