# rust-epoll-example

 [async-rust](https://github.com/night-cruise/async-rust) 中 [epoll server example](https://night-cruise.github.io/async-rust/Epoll-server-example.html) 章节的源代码。



## Run Server

克隆：

```
git clone https://github.com/night-cruise/rust-epoll-example.git
```

切换到项目目录：

```
cd rust-epoll-example
```

使用 [Cargo](https://doc.rust-lang.org/cargo/) 运行项目：

```
cargo run
```

然后我们就可以发送 `HTTP` 请求到 [http://127.0.0.1:8000](http://127.0.0.1:8000/) 了。



## Send Request

编写一个 `Python` 小脚本，使用多线程循环发送 `HTTP` 请求：

```python
import requests

from threading import Thread

with open('image.jpeg', 'rb') as f:
    FILE = f.read()


# send request to http://127.0.0.1:8000
def send_request(host, port):
    for _ in range(100):
        r = requests.post(f"http://{host}:{port}", data={'file': FILE})
        print(f"Receive response: '{r.text}' from {r.url}")


if __name__ == '__main__':
    t_lst = []
    for _ in range(4):
        t = Thread(target=send_request, args=('127.0.0.1', 8000))
        t_lst.append(t)
        t.start()

    for t in t_lst:
        t.join()

```

`server` 端的输出：

```
.....
.....
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
requests in flight: 3
got all data: 9379840 bytes
requests in flight: 3
answered from request 195
```

正如我们所看到的那样，`server` 在同时处理多个请求！
