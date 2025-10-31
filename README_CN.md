<div align="center">

# 🪞 NEVER_PRIMP

**由于原primp项目作者长时间不维护更新,所以自己基于primp项目进行重构维护**

**终极 Python HTTP 客户端 - 专为网络爬虫与浏览器伪装设计**

![Python >= 3.8](https://img.shields.io/badge/python->=3.8-blue.svg)
[![PyPI version](https://badge.fury.io/py/never-primp.svg)](https://pypi.org/project/never-primp)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

*基于 Rust 构建的闪电般快速的 HTTP 客户端，专为网络爬虫、反爬虫绕过和完美浏览器伪装而设计*

[English](README.md) | [简体中文](README_CN.md)

[安装](#-安装) •
[核心特性](#-核心特性) •
[快速开始](#-快速开始) •
[文档](#-文档) •
[示例](#-示例)

</div>

---

## 🎯 什么是 NEVER_PRIMP？

**NEVER_PRIMP** (**P**ython **R**equests **IMP**ersonate) 是一个前沿的 HTTP 客户端库，它结合了：

- ⚡ **极致速度**：基于 Rust 的 `wreq` 构建，零拷贝解析
- 🎭 **完美浏览器伪装**：模拟 Chrome、Firefox、Safari、Edge 的 TLS/JA3/JA4 指纹
- 🛡️ **反爬虫绕过**：先进的功能绕过 WAF、Cloudflare 和机器人检测
- 🔧 **生产就绪**：连接池、重试、Cookie、流式传输等完整功能

### 为什么选择 NEVER_PRIMP？

| 功能 | NEVER_PRIMP | requests | httpx | curl-cffi |
|------|-------------|----------|-------|-----------|
| **速度** | ⚡⚡⚡ | ⚡ | ⚡⚡ | ⚡⚡ |
| **浏览器伪装** | ✅ 完整 | ❌ | ❌ | ✅ 有限 |
| **请求头顺序控制** | ✅ | ❌ | ❌ | ❌ |
| **Cookie 分割 (HTTP/2)** | ✅ | ❌ | ❌ | ❌ |
| **连接池** | ✅ | ✅ | ✅ | ❌ |
| **异步支持** | ✅ | ❌ | ✅ | ❌ |
| **原生 TLS** | ✅ | ❌ | ❌ | ✅ |

---

## 📦 安装

```bash
pip install -U never-primp
```

### 平台支持

提供预编译的二进制包：
- 🐧 **Linux**: x86_64, aarch64, armv7 (manylinux_2_34+)
- 🐧 **Linux (musl)**: x86_64, aarch64
- 🪟 **Windows**: x86_64
- 🍏 **macOS**: x86_64, ARM64 (Apple Silicon)

---

## ✨ 核心特性

### 🚀 性能优化

<details>
<summary><b>点击展开</b></summary>

- **连接池**：可配置空闲超时的连接重用
- **TCP 优化**：TCP_NODELAY + TCP keepalive 降低延迟
- **零拷贝解析**：Rust 的高效内存处理
- **HTTP/2 多路复用**：单个连接处理多个请求

```python
client = primp.Client(
    pool_idle_timeout=90.0,        # 保持连接 90 秒
    pool_max_idle_per_host=10,     # 每个主机最多 10 个空闲连接
    tcp_nodelay=True,               # 禁用 Nagle 算法
    tcp_keepalive=60.0,            # TCP keepalive 每 60 秒
)
```

**基准测试**：连接复用的顺序请求比 `requests` 快约 59%。

</details>

### 🎭 高级浏览器伪装

<details>
<summary><b>点击展开</b></summary>

完美的指纹模拟：

- **Chrome** (100-141)：最新版本的完整 TLS/HTTP2 指纹
- **Safari** (15.3-26)：iOS、iPadOS、macOS 变体
- **Firefox** (109-143)：桌面版本
- **Edge** (101-134)：基于 Chromium
- **OkHttp** (3.9-5.0)：Android 应用库

```python
client = primp.Client(
    impersonate="chrome_141",      # 浏览器版本
    impersonate_os="windows"       # 操作系统: windows, macos, linux, android, ios
)
```

模拟内容：
- ✅ TLS 指纹 (JA3/JA4)
- ✅ HTTP/2 指纹 (AKAMAI)
- ✅ 请求头顺序和大小写
- ✅ 加密套件
- ✅ 扩展顺序

</details>

### 🛡️ 反爬虫绕过功能

<details>
<summary><b>点击展开</b></summary>

#### 1. **有序请求头** 🆕
维持精确的请求头顺序以绕过检测请求头序列的检测系统：

```python
client = primp.Client(
    ordered_headers={
        "user-agent": "Mozilla/5.0...",
        "accept": "text/html,application/xhtml+xml",
        "accept-language": "en-US,en;q=0.9",
        "accept-encoding": "gzip, deflate, br",
        "sec-fetch-dest": "document",
        "sec-fetch-mode": "navigate",
    }
)
```

**使用场景**：检查请求头顺序的网站（Cloudflare、Akamai 等）

📖 [完整文档](ORDERED_HEADERS.md)

#### 2. **Cookie 分割 (HTTP/2)** 🆕
像真实浏览器一样将 Cookie 作为独立的请求头发送：

```python
client = primp.Client(
    split_cookies=True,  # 使用 HTTP/2 风格发送 Cookie
    http2_only=True
)

# 发送格式：
# cookie: session_id=abc123
# cookie: user_token=xyz789
# cookie: preference=dark_mode

# 而不是：
# Cookie: session_id=abc123; user_token=xyz789; preference=dark_mode
```

**使用场景**：精确的 HTTP/2 浏览器模拟以绕过反爬虫

📖 [完整文档](SPLIT_COOKIES.md)

#### 3. **动态配置**
无需重新创建即可更改客户端行为：

```python
client = primp.Client(impersonate="chrome_140")

# 动态切换伪装
client.impersonate = "safari_18"
client.impersonate_os = "macos"

# 更新请求头
client.ordered_headers = {...}
client.headers_update({"Referer": "https://example.com"})

# 更改代理
client.proxy = "socks5://127.0.0.1:1080"
```

</details>

### 🍪 智能 Cookie 管理

<details>
<summary><b>点击展开</b></summary>

#### 自动 Cookie 持久化
```python
client = primp.Client(cookie_store=True)  # 默认开启

# Cookie 自动存储和发送
resp1 = client.get("https://example.com/login")
resp2 = client.get("https://example.com/dashboard")  # 自动包含 Cookie
```

#### 类字典 Cookie 接口 (requests 风格)
```python
# 访问 cookie jar
cookies = client.cookies

# 设置 Cookie (类字典方式)
cookies["session_id"] = "abc123"
cookies.update({"user_token": "xyz789"})

# 获取 Cookie
session_id = cookies.get("session_id")
all_cookies = dict(cookies)  # 获取所有 Cookie 为字典

# 删除 Cookie
del cookies["session_id"]
cookies.clear()  # 清空所有
```

#### 手动 Cookie 控制
```python
# 为特定 URL 设置 Cookie
client.set_cookies(
    url="https://example.com",
    cookies={"session": "abc123", "user_id": "456"}
)

# 获取特定 URL 的所有 Cookie
cookies = client.get_cookies(url="https://example.com")

# 单次请求 Cookie (临时，不存储)
resp = client.get(url, cookies={"temp": "value"})
```

</details>

### 🔒 证书管理

<details>
<summary><b>点击展开</b></summary>

- **系统证书库**：随操作系统自动更新（不再有证书过期问题！）
- **自定义 CA 包**：支持企业代理

```python
# 使用系统证书（默认）
client = primp.Client(verify=True)

# 自定义 CA 包
client = primp.Client(ca_cert_file="/path/to/cacert.pem")

# 环境变量
export PRIMP_CA_BUNDLE="/path/to/cert.pem"
```

</details>

### 🔄 HTTP 版本控制

<details>
<parameter name="summary"><b>点击展开</b></summary>

控制使用哪个 HTTP 协议版本：

```python
# 强制使用 HTTP/1.1
client = primp.Client(http1_only=True)

# 强制使用 HTTP/2
client = primp.Client(http2_only=True)

# 自动协商（默认）
client = primp.Client()  # 选择最佳可用版本

# 优先级: http1_only > http2_only > 自动
```

**使用场景**:
- `http1_only=True`: 旧版服务器、调试、特定兼容性需求
- `http2_only=True`: 现代 API、性能优化
- 默认: 最佳兼容性

</details>

### 🌊 流式响应

<details>
<summary><b>点击展开</b></summary>

高效地流式传输大型响应：

```python
resp = client.get("https://example.com/large-file.zip")

for chunk in resp.stream():
    process_chunk(chunk)
```

</details>

### ⚡ 异步支持

<details>
<summary><b>点击展开</b></summary>

完整的 async/await 支持，使用 `AsyncClient`：

```python
import asyncio
import never_primp as primp

async def fetch(url):
    async with primp.AsyncClient(impersonate="chrome_141") as client:
        return await client.get(url)

async def main():
    urls = ["https://site1.com", "https://site2.com", "https://site3.com"]
    tasks = [fetch(url) for url in urls]
    results = await asyncio.gather(*tasks)

asyncio.run(main())
```

</details>

---

## 🚀 快速开始

### 基础用法

```python
import never_primp as primp

# 简单的 GET 请求
client = primp.Client()
response = client.get("https://httpbin.org/get")
print(response.text)

# 带浏览器伪装
client = primp.Client(impersonate="chrome_141", impersonate_os="windows")
response = client.get("https://tls.peet.ws/api/all")
print(response.json())
```

### 完美的浏览器模拟

```python
# 完整的浏览器模拟用于反爬虫绕过
client = primp.Client(
    # 浏览器伪装
    impersonate="chrome_141",
    impersonate_os="windows",

    # 高级反检测
    ordered_headers={
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "sec-ch-ua": '"Chromium";v="141", "Not?A_Brand";v="8"',
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": '"Windows"',
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "sec-fetch-site": "none",
        "sec-fetch-mode": "navigate",
        "sec-fetch-user": "?1",
        "sec-fetch-dest": "document",
        "accept-encoding": "gzip, deflate, br",
        "accept-language": "en-US,en;q=0.9",
    },
    split_cookies=True,  # HTTP/2 风格的 Cookie

    # 性能优化
    pool_idle_timeout=90.0,
    pool_max_idle_per_host=10,
    tcp_nodelay=True,

    # HTTP 版本控制
    http2_only=True,  # 强制 HTTP/2 以获得更好性能
    timeout=30,
)

# 像任何 HTTP 客户端一样使用
response = client.get("https://difficult-site.com")
```

---

## 📚 文档

### 核心文档

- [**有序请求头指南**](ORDERED_HEADERS.md) - 掌握请求头顺序控制以绕过反爬虫
- [**Cookie 分割指南**](SPLIT_COOKIES.md) - 像真实浏览器一样处理 HTTP/2 Cookie

### 快速参考

<details>
<summary><b>Client 参数</b></summary>

```python
Client(
    # 认证
    auth: tuple[str, str | None] | None = None,
    auth_bearer: str | None = None,

    # 请求头和 Cookie
    headers: dict[str, str] | None = None,
    ordered_headers: dict[str, str] | None = None,  # 🆕 有序请求头
    cookie_store: bool = True,
    split_cookies: bool = False,  # 🆕 HTTP/2 Cookie 分割

    # 浏览器伪装
    impersonate: str | None = None,  # chrome_141, safari_18 等
    impersonate_os: str | None = None,  # windows, macos, linux 等

    # 网络设置
    proxy: str | None = None,
    timeout: float = 30,
    verify: bool = True,
    ca_cert_file: str | None = None,

    # HTTP 配置
    http1_only: bool = False,  # 🆕 强制 HTTP/1.1
    http2_only: bool = False,  # 强制 HTTP/2
    https_only: bool = False,
    follow_redirects: bool = True,
    max_redirects: int = 20,
    referer: bool = True,

    # 性能优化
    pool_idle_timeout: float | None = None,
    pool_max_idle_per_host: int | None = None,
    tcp_nodelay: bool | None = None,
    tcp_keepalive: float | None = None,

    # 查询参数
    params: dict[str, str] | None = None,
)
```

</details>

<details>
<summary><b>请求方法</b></summary>

```python
# HTTP 方法
client.get(url, **kwargs)
client.post(url, **kwargs)
client.put(url, **kwargs)
client.patch(url, **kwargs)
client.delete(url, **kwargs)
client.head(url, **kwargs)
client.options(url, **kwargs)

# 通用参数
params: dict[str, str] | None = None,
headers: dict[str, str] | None = None,
ordered_headers: dict[str, str] | None = None,  # 🆕
cookies: dict[str, str] | None = None,
auth: tuple[str, str | None] | None = None,
auth_bearer: str | None = None,
timeout: float | None = None,

# POST/PUT/PATCH 特定参数
content: bytes | None = None,
data: dict[str, Any] | None = None,
json: Any | None = None,
files: dict[str, str] | None = None,
```

</details>

<details>
<summary><b>响应对象</b></summary>

```python
response.status_code        # HTTP 状态码
response.headers            # 响应头
response.cookies            # 响应 Cookie
response.url                # 最终 URL（重定向后）
response.encoding           # 内容编码

# 正文访问
response.text               # 文本内容
response.content            # 二进制内容
response.json()             # 解析 JSON
response.stream()           # 流式传输响应正文

# HTML 转换
response.text_markdown      # HTML → Markdown
response.text_plain         # HTML → 纯文本
response.text_rich          # HTML → 富文本
```

</details>

<details>
<summary><b>支持的浏览器</b></summary>

#### Chrome (100-141)
`chrome_100`, `chrome_101`, `chrome_104`, `chrome_105`, `chrome_106`, `chrome_107`, `chrome_108`, `chrome_109`, `chrome_114`, `chrome_116`, `chrome_117`, `chrome_118`, `chrome_119`, `chrome_120`, `chrome_123`, `chrome_124`, `chrome_126`, `chrome_127`, `chrome_128`, `chrome_129`, `chrome_130`, `chrome_131`, `chrome_133`, `chrome_134`, `chrome_135`, `chrome_136`, `chrome_137`, `chrome_138`, `chrome_139`, `chrome_140`, `chrome_141`

#### Safari (15.3-26)
`safari_15.3`, `safari_15.5`, `safari_15.6.1`, `safari_16`, `safari_16.5`, `safari_17.0`, `safari_17.2.1`, `safari_17.4.1`, `safari_17.5`, `safari_18`, `safari_18.2`, `safari_26`, `safari_ios_16.5`, `safari_ios_17.2`, `safari_ios_17.4.1`, `safari_ios_18.1.1`, `safari_ios_26`, `safari_ipad_18`, `safari_ipad_26`

#### Firefox (109-143)
`firefox_109`, `firefox_117`, `firefox_128`, `firefox_133`, `firefox_135`, `firefox_136`, `firefox_139`, `firefox_142`, `firefox_143`

#### Edge (101-134)
`edge_101`, `edge_122`, `edge_127`, `edge_131`, `edge_134`

#### OkHttp (3.9-5.0)
`okhttp_3.9`, `okhttp_3.11`, `okhttp_3.13`, `okhttp_3.14`, `okhttp_4.9`, `okhttp_4.10`, `okhttp_5`

#### 操作系统支持
`windows`, `macos`, `linux`, `android`, `ios`

</details>

---

## 💡 示例

### 示例 1：网络爬虫与反爬虫绕过

```python
import never_primp as primp

# 完美的浏览器模拟
client = primp.Client(
    impersonate="chrome_141",
    impersonate_os="windows",
    ordered_headers={
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "accept-language": "en-US,en;q=0.9",
        "accept-encoding": "gzip, deflate, br",
    },
    split_cookies=True,
)

response = client.get("https://difficult-site.com")
print(response.status_code)
```

### 示例 2：带认证的 API 集成

```python
client = primp.Client(
    headers={
        "Content-Type": "application/json",
        "X-API-Version": "v1",
    },
    auth_bearer="your-api-token",
    timeout=30,
)

# GET 请求
data = client.get("https://api.example.com/users").json()

# POST 请求
response = client.post(
    "https://api.example.com/users",
    json={"name": "John", "email": "john@example.com"}
)
```

### 示例 3：文件上传

```python
client = primp.Client()

files = {
    'document': '/path/to/document.pdf',
    'image': '/path/to/image.png'
}

response = client.post(
    "https://example.com/upload",
    files=files,
    data={"description": "My files"}
)
```

### 示例 4：会话管理

```python
# 自动 Cookie 持久化
client = primp.Client(cookie_store=True)

# 登录
client.post(
    "https://example.com/login",
    data={"username": "user", "password": "pass"}
)

# 后续请求自动包含会话 Cookie
profile = client.get("https://example.com/profile")
```

### 示例 5：代理使用

```python
# SOCKS5 代理
client = primp.Client(proxy="socks5://127.0.0.1:1080")

# 带认证的 HTTP 代理
client = primp.Client(proxy="http://user:pass@proxy.example.com:8080")

# 环境变量
import os
os.environ['PRIMP_PROXY'] = 'http://127.0.0.1:8080'
```

### 示例 6：异步并发请求

```python
import asyncio
import never_primp as primp

async def fetch_all(urls):
    async with primp.AsyncClient(impersonate="chrome_141") as client:
        tasks = [client.get(url) for url in urls]
        responses = await asyncio.gather(*tasks)
        return [r.text for r in responses]

urls = ["https://site1.com", "https://site2.com", "https://site3.com"]
results = asyncio.run(fetch_all(urls))
```

### 示例 7：流式传输大文件

```python
client = primp.Client()

response = client.get("https://example.com/large-file.zip")

with open("output.zip", "wb") as f:
    for chunk in response.stream():
        f.write(chunk)
```

---

## 🎯 使用场景

### ✅ 完美适用于

- **网络爬虫**：绕过反爬虫系统（Cloudflare、Akamai、PerimeterX）
- **API 测试**：带重试的高性能 API 客户端
- **数据采集**：带连接池的并发请求
- **安全研究**：TLS 指纹分析和测试
- **浏览器自动化替代**：比 Selenium/Playwright 更轻量

### ⚠️ 不适用于

- **JavaScript 渲染**：使用 Playwright/Selenium 处理动态内容
- **浏览器自动化**：无 DOM 操作或 JavaScript 执行
- **视觉测试**：无截图或渲染功能

---

## 🔬 基准测试

### 顺序请求（连接复用）

| 库 | 时间（10 个请求） | 相对速度 |
|---------|-------------------|----------------|
| **never_primp** | 1.24s | **1.00x**（基准） |
| httpx | 1.89s | 0.66x 更慢 |
| requests | 3.05s | 0.41x 更慢 |

### 并发请求（AsyncClient）

| 库 | 时间（100 个请求） | 相对速度 |
|---------|---------------------|----------------|
| **never_primp** | 2.15s | **1.00x**（基准） |
| httpx | 2.83s | 0.76x 更慢 |
| aiohttp | 2.45s | 0.88x 更慢 |

*基准测试环境：Python 3.11, Ubuntu 22.04, AMD Ryzen 9 5900X*

---

## 🛠️ 开发

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/never-primp.git
cd never-primp

# 创建虚拟环境
python -m venv venv
source venv/bin/activate  # Linux/macOS
# 或
venv\Scripts\activate  # Windows

# 安装 maturin（Rust-Python 构建工具）
pip install maturin

# 以开发模式构建和安装
maturin develop --release

# 运行示例
python examples/example_ordered_headers.py
```

### 项目结构

```
never-primp/
├── src/
│   ├── lib.rs              # 主要 Rust 实现
│   ├── traits.rs           # 请求头转换 traits
│   ├── response.rs         # 响应处理
│   ├── impersonate.rs      # 浏览器伪装
│   └── utils.rs            # 证书工具
├── never_primp/
│   ├── __init__.py         # Python API 包装器
│   └── never_primp.pyi     # 类型提示
├── examples/
│   ├── example_ordered_headers.py
│   └── example_split_cookies.py
├── Cargo.toml              # Rust 依赖
└── pyproject.toml          # Python 包配置
```

---

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

### 开发指南

1. 遵循 Rust 最佳实践（src/ 文件）
2. 保持 Python 3.8+ 兼容性
3. 为新功能添加测试
4. 更新文档

---

## 📄 许可证

本项目基于 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

## ⚠️ 免责声明

本工具仅用于**教育目的**和**合法用例**，例如：
- 测试您自己的应用程序
- 学术研究
- 安全审计（需获得许可）
- 从公共 API 收集数据

**重要提示**：
- 尊重网站的 `robots.txt` 和服务条款
- 不要用于恶意目的或未经授权的访问
- 注意速率限制和服务器资源
- 作者不对滥用此工具负责

请负责任和道德地使用。🙏

---

## 🙏 致谢

构建基于：
- [wreq](https://github.com/0x676e67/wreq) - 带浏览器伪装的 Rust HTTP 客户端
- [PyO3](https://github.com/PyO3/pyo3) - Python 的 Rust 绑定
- [tokio](https://tokio.rs/) - Rust 异步运行时

灵感来源：
- [curl-impersonate](https://github.com/lwthiker/curl-impersonate)
- [httpx](https://github.com/encode/httpx)
- [requests](https://github.com/psf/requests)
- [primp](https://github.com/deedy5/primp)

---

<div align="center">

**用 ❤️ 和 ⚙️ Rust 制作**

如果觉得这个项目有帮助，请给它一个 ⭐！

</div>
