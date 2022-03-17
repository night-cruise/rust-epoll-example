//! 处理 http 请求

use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::epoll::{
    close, listener_read_event, listener_write_event, modify_interest, remove_interest,
};
use crate::{AsRawFd, RawFd};

// 返回的响应为固定的 HTTP 文本
const HTTP_RESP: &[u8] = br#"HTTP/1.1 200 OK
content-type: text/html
content-length: 28

Hello! I am an epoll server."#;

/// 请求上下文，用于处理 HTTP 请求
#[derive(Debug)]
pub struct RequestContext {
    /// 与客户端建立的连接的 stream 流
    pub stream: TcpStream,
    /// 收到的 HTTP 请求的 content-length 的值
    pub content_length: usize,
    /// 收到的 HTTP 请求的数据写入的缓冲区
    pub buf: Vec<u8>,
}

impl RequestContext {
    /// 创建一个请求上下文实例
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buf: Vec::new(),
            content_length: 0,
        }
    }

    /// 从 stream 流中读取数据
    pub fn read_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        // 从 stream 流中读取数据写入到 buf 中
        match self.stream.read(&mut buf) {
            Ok(_) => {
                if let Ok(data) = std::str::from_utf8(&buf) {
                    // 解析并且设置读取到的 HTTP 请求的 content-length 字段的值
                    self.parse_and_set_content_length(data);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => {
                return Err(e);
            }
        };

        // 将读取的数据扩展到 RequestContext 的 buf 中
        self.buf.extend_from_slice(&buf);
        // 如果 buf 中的数据长度大于等于 content-length，说明从客户端发送的 HTTP 请求已经读取完毕
        if self.buf.len() >= self.content_length {
            println!("got all data: {} bytes", self.buf.len());
            // 将在 stream 上监听的事件修改为写事件
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_write_event(key))?;
        } else {
            // 将在 stream 上监听的事件修改为读事件，继续读取剩下的 HTTP 请求
            modify_interest(epoll_fd, self.stream.as_raw_fd(), listener_read_event(key))?;
        }

        Ok(())
    }

    /// 解析并且设置读取到的 HTTP 请求的 content-length 字段的值
    pub fn parse_and_set_content_length(&mut self, data: &str) {
        if data.contains("HTTP") {
            if let Some(content_length) = data
                .lines()
                .find(|l| l.to_lowercase().starts_with("content-length: "))
            {
                if let Some(len) = content_length
                    .to_lowercase()
                    .strip_prefix("content-length: ")
                {
                    self.content_length = len.parse::<usize>().expect("content-length is valid");
                    println!("set content length: {} bytes", self.content_length);
                }
            }
        }
    }

    /// 将要返回的 HTTP 数据写入到 stream 流中
    pub fn write_cb(&mut self, key: u64, epoll_fd: RawFd) -> io::Result<()> {
        match self.stream.write(HTTP_RESP) {
            Ok(_) => println!("answered from request {}", key),
            Err(e) => eprintln!("could not answer to request {}, {}", key, e),
        };
        // 关闭 stream 流
        self.stream.shutdown(std::net::Shutdown::Both)?;
        let fd = self.stream.as_raw_fd();
        // 移除在 epoll 中注册的文件描述符 fd
        remove_interest(epoll_fd, fd)?;
        // 关闭文件描述符 fd
        close(fd);

        Ok(())
    }
}
