use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

use rust_epoll_example::epoll_syscall::{add_interest, epoll_create, listener_read_event, modify_interest};
use rust_epoll_example::http_handle::RequestContext;
use rust_epoll_example::syscall;


fn main() -> io::Result<()> {
    // 存储请求上下文实例，key用来区分不同的请求上下文
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();
    // 存储就绪的event序列
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    // 一个key对应一个请求上下文实例(并且对应其中stream关联的事件)，当key是100时对应listener文件描述符
    let mut key = 100;

    // 绑定IP地址
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    // 将socket设置为非阻塞
    listener.set_nonblocking(true)?;
    // 获取listener的文件描述符
    let listener_fd = listener.as_raw_fd();

    // 创建epoll实例，返回epoll文件描述符
    let epoll_fd = epoll_create().expect("can create epoll queue");
    // 在epoll实例中注册listener文件描述符，并关联一个读事件
    add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

    loop {
        println!("requests in flight: {}", request_contexts.len());
        events.clear();
        // 将就绪的事件添加到events数组中，返回就绪的事件数量
        let res = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000 as libc::c_int,
        )) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: {}", e),
        };

        // safe  as long as the kernel does nothing wrong - copied from mio
        // 根据就绪的事件数量设置events数组的长度
        unsafe { events.set_len(res as usize) };

        // 遍历处理就绪的事件
        for ev in &events {
            match ev.u64 {
                // 100说明是listener上的事件就绪，有新的socket连接到来
                100 => {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            stream.set_nonblocking(true)?;
                            println!("new client: {}", addr);
                            key += 1;
                            // 在epoll中注册stream文件描述符
                            add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                            // 创建一个请求上下文实例，并保存到request_contexts中
                            request_contexts.insert(key, RequestContext::new(stream));
                        }
                        Err(e) => eprintln!("couldn't accept: {}", e),
                    };
                    // 修改listener关联的事件为读事件
                    modify_interest(epoll_fd, listener_fd, listener_read_event(100))?;
                }
                // key!=100，说明是其他文件描述符上的事件就绪了
                key => {
                    let mut to_delete = None;
                    // 获取该key对应的请求上下文实例
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;
                        // 模式匹配，判断就绪的是读事件还是写事件
                        match events {
                            // 读事件就绪
                            v if v as i32 & libc::EPOLLIN == libc::EPOLLIN => {
                                // 读取请求数据
                                context.read_cb(key, epoll_fd)?;
                            }
                            // 写事件就绪
                            v if v as i32 & libc::EPOLLOUT == libc::EPOLLOUT => {
                                // 写入返回数据
                                context.write_cb(key, epoll_fd)?;
                                to_delete = Some(key);
                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }
                    // 如果是写事件就绪，那么就删除这个key对应的请求上下文实例
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            }
        }
    }
}