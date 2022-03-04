//! 封装 epoll 系统调用

use std::io;

use crate::RawFd;
use crate::syscall;

const READ_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
const WRITE_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLOUT;

/// 包装epoll_create，创建一个epoll实例
pub fn epoll_create() -> io::Result<RawFd> {
    // 创建一个epoll实例，返回epoll对象的文件描述符fd
    let fd = syscall!(epoll_create1(0))?;

    // 返回与 fd 关联的close_an_exec标志
    // close_on_exe用于确定在系统调用execve()是否需要关闭文件描述符
    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {

        // 设置 fd 关联的close_on_exec标志
        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
    }

    Ok(fd)
}

/// 包装epoll_ctl api，注册文件描述符
pub fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    // epoll_fd是epoll实例的的文件描述符
    // fd是要注册的目标文件描述符
    // event是与fd相关联的事件
    // libc::EPOLL_CTL_ADD表示注册fd并把event与fd关联起来
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;
    Ok(())
}

/// 包装epoll_ctl api，修改文件描述符
pub fn modify_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    // epoll_fd是epoll实例的的文件描述符
    // fd是要修改目标文件描述符
    // event是与fd相关联的事件
    // libc::EPOLL_CTL_MOD表示修改与fd关键的event
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event))?;
    Ok(())
}

/// 包装epoll_ctl api，删除文件描述符
pub fn remove_interest(epoll_fd: RawFd, fd: RawFd) -> io::Result<()> {
    // epoll_fd是epoll实例的的文件描述符
    // fd是要删除的目标文件描述符
    // libc::EPOLL_CTL_DEL表示删除fd
    // std::ptr::null_mut()表示将event置为null
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_DEL,
        fd,
        std::ptr::null_mut()
    ))?;
    Ok(())
}

/// 关闭文件描述符fd
pub fn close(fd: RawFd) {
    let _ = syscall!(close(fd));
}


/// 创建一个读事件
pub fn listener_read_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: READ_FLAGS as u32,
        u64: key,
    }
}

/// 创建一个写事件
pub fn listener_write_event(key: u64) -> libc::epoll_event {
    libc::epoll_event {
        events: WRITE_FLAGS as u32,
        u64: key,
    }
}