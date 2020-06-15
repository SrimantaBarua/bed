// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

macro_rules! debug {
    ($($arg:tt)*) => (eprintln!("[DEBUG]: {}:{}: {}",
                                module_path!(), line!(), format_args!($($arg)*)))
}

macro_rules! error {
    ($($arg:tt)*) => (eprintln!("[ERROR]: {}:{}: {}",
                                module_path!(), line!(), format_args!($($arg)*)))
}
