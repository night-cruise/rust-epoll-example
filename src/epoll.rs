//! 封装 epoll 系统调用

use std::io;

use crate::{syscall, RawFd};

const READ_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLIN;
const WRITE_FLAGS: i32 = libc::EPOLLONESHOT | libc::EPOLLOUT;

/// 包装epoll_create，创建一个epoll实例
pub fn epoll_create() -> io::Result<RawFd> {
    // 创建一个epoll实例，返回epoll对象的文件描述符fd
    let fd = syscall!(epoll_create1(0))?;

    // fcntl(fd, libc::F_GETFD) 函数返回与 fd 关联的 close_on_exec 标志
    // close_on_exec 用于确定在系统调用 execve() 后是否需要关闭文件描述符
    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
        // 设置在系统调用 execve() 后关闭文件描述符 fd
        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
    }

    Ok(fd)
}

/// 包装 epoll_ctl，注册文件描述符和事件
pub fn add_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    // epoll_fd 是 epoll 实例的的文件描述符
    // fd 是要注册的目标文件描述符
    // event 是要在 fd 上监听的事件
    // libc::EPOLL_CTL_ADD 表示添加一个需要监视的文件描述符
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event))?;

    Ok(())
}

/// 包装 epoll_ctl，修改文件描述符
pub fn modify_interest(epoll_fd: RawFd, fd: RawFd, mut event: libc::epoll_event) -> io::Result<()> {
    // epoll_fd 是 epoll 实例的的文件描述符
    // fd 是要修改目标文件描述符
    // event 是要在 fd 上监听的事件
    // libc::EPOLL_CTL_MOD 表示修改文件描述符 fd
    syscall!(epoll_ctl(epoll_fd, libc::EPOLL_CTL_MOD, fd, &mut event))?;

    Ok(())
}

/// 包装 epoll_ctl，删除文件描述符
pub fn remove_interest(epoll_fd: RawFd, fd: RawFd) -> io::Result<()> {
    // epoll_fd 是 epoll 实例的的文件描述符
    // fd 是要删除的目标文件描述符
    // libc::EPOLL_CTL_DEL 表示要删除文件描述符 fd
    syscall!(epoll_ctl(
        epoll_fd,
        libc::EPOLL_CTL_DEL,
        fd,
        std::ptr::null_mut() // 将监听的 event 设置为空
    ))?;

    Ok(())
}

/// 关闭文件描述符 fd
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
