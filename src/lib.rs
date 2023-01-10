pub struct User {
    login: String,
    pasword: String,
}

pub enum Status {
    OK,
    ERROR,
}

pub fn runprog_safemode(user: Option<&User>, program: &str) -> Status {
    Status::OK
}

pub fn back_to_normalmode() -> Status {
    Status::ERROR
}
