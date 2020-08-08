// (C) 2020 Srimanta Barua <srimanta.barua1@gmail.com>

#[cfg(target_os = "linux")]
mod linux_x11;

#[cfg(target_os = "linux")]
pub(crate) use linux_x11::*;
