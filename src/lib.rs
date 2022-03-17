use std::os::unix::io::{AsRawFd, RawFd};

pub mod epoll;
pub mod http;

/// 这是一个宏，用于方便地调用 epoll 的 API: epoll_create、epoll_ctl、epoll_wait
///
/// 以 epoll_wait 为例：
/// ```
/// syscall!(epoll_wait(
///             epoll_fd,
///             events.as_mut_ptr() as *mut libc::epoll_event,
///             1024,
///             1000 as libc::c_int,
/// ))
/// ```
/// 展开后的代码为：
/// ```
/// let _ = {
///     let res = unsafe {
///         libc::epoll_wait(
///             epoll_fd,
///             events.as_mut_ptr() as *mut libc::epoll_event,
///             1024,
///             1000 as libc::c_int,
///         )
///     };
///     if res == -1 {
///         Err(std::io::Error::last_os_error())
///     } else {
///         Ok(res)
///     }
/// };
/// ```
#[macro_export]
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}
