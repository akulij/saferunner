#[allow(dead_code)]

pub fn is_wow64() -> bool {
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::wow64apiset::IsWow64Process;

    unsafe {
        let mut is_wow = 0;
        if IsWow64Process(GetCurrentProcess(), &mut is_wow) == 0 {
            return false;
        }

        is_wow != 0
    }
}

pub fn encode_str(s: &str) -> Vec<u16> {
    let mut v= s.encode_utf16().collect::<Vec<u16>>();
    v.push(0);
    v
}

pub fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).expect("error creating cstring")
}

pub fn nullptr<T>() -> *mut T {
    std::ptr::null::<T>().cast_mut()
}

pub fn random_code(lenght: u32) -> String {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    let alphabet = ('a'..='z').into_iter().collect::<Vec<char>>();

    let mut rand_gen = thread_rng();
    (0..lenght)
        .map(|_| {
            alphabet
                .choose(&mut rand_gen)
                .expect("Can't get char from alphabet")
        })
        .collect::<String>()
}

pub fn cast_ptr<T, O>(ptr: *mut T) -> *mut O {
    unsafe {
        std::mem::transmute(ptr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_random_code() {
        let code = random_code(4);

        assert_eq!(code.len(), 4);
    }
}
