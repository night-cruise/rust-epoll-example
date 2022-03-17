use std::collections::HashMap;
use std::io;
use std::net::TcpListener;
use std::os::unix::io::AsRawFd;

use rust_epoll_example::epoll::{add_interest, epoll_create, listener_read_event, modify_interest};
use rust_epoll_example::http::RequestContext;
use rust_epoll_example::syscall;

fn main() -> io::Result<()> {
    // 存储 RequestContext 实例，key 用来区分不同的 RequestContext
    let mut request_contexts: HashMap<u64, RequestContext> = HashMap::new();
    // 存储就绪的 event
    let mut events: Vec<libc::epoll_event> = Vec::with_capacity(1024);
    // key 对应 epoll_event 中的 u64 字段，用于区分文件描述、RequestContext
    let mut key = 100;

    // 创建一个 listener，并监听 8000 端口
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    // 将 socket 设置为非阻塞
    listener.set_nonblocking(true)?;
    // 获取 listener 对应文件描述符
    let listener_fd = listener.as_raw_fd();

    // 创建 epoll 实例，返回 epoll 文件描述符
    let epoll_fd = epoll_create().expect("can create epoll queue");
    // 在 epoll 实例中注册 listener 文件描述符，并监听读事件
    // key 等于 100，对应 listener 文件描述符
    add_interest(epoll_fd, listener_fd, listener_read_event(key))?;

    loop {
        println!("requests in flight: {}", request_contexts.len());
        events.clear();
        // 将就绪的事件添加到 events vec 中，返回就绪的事件数量
        let res = match syscall!(epoll_wait(
            epoll_fd,
            events.as_mut_ptr() as *mut libc::epoll_event,
            1024,
            1000,
        )) {
            Ok(v) => v,
            Err(e) => panic!("error during epoll wait: {}", e),
        };

        // safe  as long as the kernel does nothing wrong - copied from mio
        // 根据就绪的事件数量设置 events vec 的长度
        unsafe { events.set_len(res as usize) };

        // 遍历处理就绪的事件
        for ev in &events {
            match ev.u64 {
                // key = 100 说明是在 listener fd 上监听的读事件就绪了
                100 => {
                    match listener.accept() {
                        // stream 是与客户端建立的连接的 stream 流
                        Ok((stream, addr)) => {
                            // 设置为非阻塞
                            stream.set_nonblocking(true)?;
                            // 有一个新的连接来了
                            println!("new client: {}", addr);
                            key += 1;
                            // 在 epoll 中注册 stream 文件描述符，并监听读事件
                            add_interest(epoll_fd, stream.as_raw_fd(), listener_read_event(key))?;
                            // 创建一个 RequestContext，并保存到 request_contexts 中
                            request_contexts.insert(key, RequestContext::new(stream));
                            // 上面使用的 key，用来区分不同的文件描述符和 RequestContext
                        }
                        Err(e) => eprintln!("couldn't accept: {}", e),
                    };
                    // 修改在 listener fd 上监听的的事件为读事件（继续等待新的连接到来）
                    modify_interest(epoll_fd, listener_fd, listener_read_event(100))?;
                }
                // key != 100，说明是其他的 fd 上监听的事件就绪了
                key => {
                    let mut to_delete = None;
                    // 获取这个 key 对应的 RequestContext
                    if let Some(context) = request_contexts.get_mut(&key) {
                        let events: u32 = ev.events;
                        // 匹配就绪的事件是读事件还是写事件
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
                                // 返回数据后，就删除对应的 RequestContext，当客户端再次发起请求时会建立新的连接，创建新的 RequestContext
                                to_delete = Some(key);
                            }
                            v => println!("unexpected events: {}", v),
                        };
                    }
                    // 写事件处理完毕，删除对应的 RequestContext
                    if let Some(key) = to_delete {
                        request_contexts.remove(&key);
                    }
                }
            }
        }
    }
}
