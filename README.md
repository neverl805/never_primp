<div align="center">

# 🪞 NEVER_PRIMP

**Since the original primp project author did not maintain updates for a long time, he refactored and maintained based on the primp project**

**The Ultimate Python HTTP Client for Web Scraping & Browser Impersonation**

![Python >= 3.8](https://img.shields.io/badge/python->=3.8-blue.svg)
[![PyPI version](https://badge.fury.io/py/never-primp.svg)](https://pypi.org/project/never-primp)
[![Downloads](https://static.pepy.tech/badge/never-primp/week)](https://pepy.tech/project/never-primp)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)

*Lightning-fast HTTP client built with Rust, designed for web scraping, anti-bot bypass, and perfect browser impersonation*

[English](README.md) | [简体中文](README_CN.md)

[Installation](#-installation) •
[Key Features](#-key-features) •
[Quick Start](#-quick-start) •
[Documentation](#-documentation) •
[Examples](#-examples)

</div>

---

## 🎯 What is NEVER_PRIMP?

**NEVER_PRIMP** (**P**ython **R**equests **IMP**ersonate) is a cutting-edge HTTP client library that combines:

- ⚡ **Blazing Speed**: Built on Rust's `wreq` with zero-copy parsing
- 🎭 **Perfect Browser Impersonation**: Mimic Chrome, Firefox, Safari, Edge down to TLS/JA3/JA4 fingerprints
- 🛡️ **Anti-Bot Bypass**: Advanced features for bypassing WAF, Cloudflare, and bot detection
- 🔧 **Production-Ready**: Connection pooling, retries, cookies, streaming, and more

### Why Choose NEVER_PRIMP?

| Feature | NEVER_PRIMP | requests | httpx | curl-cffi |
|---------|-------------|----------|-------|-----------|
| **Speed** | ⚡⚡⚡ | ⚡ | ⚡⚡ | ⚡⚡ |
| **Browser Impersonation** | ✅ Full | ❌ | ❌ | ✅ Limited |
| **Header Order Control** | ✅ | ❌ | ❌ | ❌ |
| **Cookie Splitting (HTTP/2)** | ✅ | ❌ | ❌ | ❌ |
| **Connection Pooling** | ✅ | ✅ | ✅ | ❌ |
| **Async Support** | ✅ | ❌ | ✅ | ❌ |
| **Native TLS** | ✅ | ❌ | ❌ | ✅ |

---

## 📦 Installation

```bash
pip install -U never-primp
```

### Platform Support

Precompiled wheels available for:
- 🐧 **Linux**: x86_64, aarch64, armv7 (manylinux_2_34+)
- 🐧 **Linux (musl)**: x86_64, aarch64
- 🪟 **Windows**: x86_64
- 🍏 **macOS**: x86_64, ARM64 (Apple Silicon)

---

## ✨ Key Features

### 🚀 Performance Optimized

<details>
<summary><b>Click to expand</b></summary>

- **Connection Pooling**: Reuse connections with configurable idle timeout
- **TCP Optimization**: TCP_NODELAY + TCP keepalive for lower latency
- **Zero-Copy Parsing**: Rust's efficient memory handling
- **HTTP/2 Multiplexing**: Multiple requests over single connection

```python
client = primp.Client(
    pool_idle_timeout=90.0,        # Keep connections alive 90s
    pool_max_idle_per_host=10,     # Max 10 idle connections per host
    tcp_nodelay=True,               # Disable Nagle's algorithm
    tcp_keepalive=60.0,            # TCP keepalive every 60s
)
```

**Benchmark**: ~59% faster than `requests` for sequential requests with connection reuse.

</details>

### 🎭 Advanced Browser Impersonation

<details>
<summary><b>Click to expand</b></summary>

Perfect fingerprint mimicry for:

- **Chrome** (100-141): Latest versions with full TLS/HTTP2 fingerprints
- **Safari** (15.3-26): iOS, iPadOS, macOS variants
- **Firefox** (109-143): Desktop versions
- **Edge** (101-134): Chromium-based
- **OkHttp** (3.9-5.0): Android application library

```python
client = primp.Client(
    impersonate="chrome_141",      # Browser version
    impersonate_os="windows"       # OS: windows, macos, linux, android, ios
)
```

Impersonates:
- ✅ TLS fingerprint (JA3/JA4)
- ✅ HTTP/2 fingerprint (AKAMAI)
- ✅ Header order and casing
- ✅ Cipher suites
- ✅ Extension order

</details>

### 🛡️ Anti-Bot Bypass Features

<details>
<summary><b>Click to expand</b></summary>

#### 1. **Ordered Headers** 🆕
Maintain exact header order to bypass detection systems that check header sequence:

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

**Use Case**: Websites checking header order (Cloudflare, Akamai, etc.)

📖 [Full Documentation](ORDERED_HEADERS.md)

#### 2. **Split Cookies (HTTP/2)** 🆕
Send cookies as separate headers like real browsers:

```python
client = primp.Client(
    split_cookies=True,  # Send cookies in HTTP/2 style
    http2_only=True
)

# Sends:
# cookie: session_id=abc123
# cookie: user_token=xyz789
# cookie: preference=dark_mode

# Instead of:
# Cookie: session_id=abc123; user_token=xyz789; preference=dark_mode
```

**Use Case**: Precise HTTP/2 browser simulation for anti-bot bypass

📖 [Full Documentation](SPLIT_COOKIES.md)

#### 3. **Dynamic Configuration**
Change client behavior without recreation:

```python
client = primp.Client(impersonate="chrome_140")

# Switch impersonation dynamically
client.impersonate = "safari_18"
client.impersonate_os = "macos"

# Update headers
client.ordered_headers = {...}
client.headers_update({"Referer": "https://example.com"})

# Change proxy
client.proxy = "socks5://127.0.0.1:1080"
```

</details>

### 🍪 Smart Cookie Management

<details>
<summary><b>Click to expand</b></summary>

#### Automatic Cookie Persistence
```python
client = primp.Client(cookie_store=True)  # Default

# Cookies automatically stored and sent
resp1 = client.get("https://example.com/login")
resp2 = client.get("https://example.com/dashboard")  # Cookies included
```

#### Manual Cookie Control
```python
# Set cookies
client.set_cookies(
    url="https://example.com",
    cookies={"session": "abc123", "user_id": "456"}
)

# Get cookies
cookies = client.get_cookies(url="https://example.com")

# Per-request cookies
resp = client.get(url, cookies={"temp": "value"})
```

</details>

### 🔒 Certificate Management

<details>
<summary><b>Click to expand</b></summary>

- **System Certificate Store**: Auto-updated with OS (no more expiration issues!)
- **Custom CA Bundle**: Support for corporate proxies

```python
# Use system certificates (default)
client = primp.Client(verify=True)

# Custom CA bundle
client = primp.Client(ca_cert_file="/path/to/cacert.pem")

# Environment variable
export PRIMP_CA_BUNDLE="/path/to/cert.pem"
```

</details>

### 🔄 Retry Mechanism

<details>
<summary><b>Click to expand</b></summary>

Automatic retries with exponential backoff:

```python
client = primp.Client(
    retry_count=3,        # Retry up to 3 times
    retry_backoff=1.0,    # 1 second backoff between retries
)
```

Handles transient failures gracefully.

</details>

### 🌊 Streaming Responses

<details>
<summary><b>Click to expand</b></summary>

Stream large responses efficiently:

```python
resp = client.get("https://example.com/large-file.zip")

for chunk in resp.stream():
    process_chunk(chunk)
```

</details>

### ⚡ Async Support

<details>
<summary><b>Click to expand</b></summary>

Full async/await support with `AsyncClient`:

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

## 🚀 Quick Start

### Basic Usage

```python
import never_primp as primp

# Simple GET request
client = primp.Client()
response = client.get("https://httpbin.org/get")
print(response.text)

# With browser impersonation
client = primp.Client(impersonate="chrome_141", impersonate_os="windows")
response = client.get("https://tls.peet.ws/api/all")
print(response.json())
```

### Perfect Browser Simulation

```python
# Complete browser simulation for anti-bot bypass
client = primp.Client(
    # Browser impersonation
    impersonate="chrome_141",
    impersonate_os="windows",

    # Advanced anti-detection
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
    split_cookies=True,  # HTTP/2 style cookies

    # Performance optimization
    pool_idle_timeout=90.0,
    pool_max_idle_per_host=10,
    tcp_nodelay=True,

    # Reliability
    retry_count=3,
    retry_backoff=1.0,
    timeout=30,
)

# Use like any HTTP client
response = client.get("https://difficult-site.com")
```

---

## 📚 Documentation

### Core Documentation

- [**Ordered Headers Guide**](ORDERED_HEADERS.md) - Master header order control for anti-bot bypass
- [**Split Cookies Guide**](SPLIT_COOKIES.md) - HTTP/2 cookie handling like real browsers

### Quick References

<details>
<summary><b>Client Parameters</b></summary>

```python
Client(
    # Authentication
    auth: tuple[str, str | None] | None = None,
    auth_bearer: str | None = None,

    # Headers & Cookies
    headers: dict[str, str] | None = None,
    ordered_headers: dict[str, str] | None = None,  # 🆕 Ordered headers
    cookie_store: bool = True,
    split_cookies: bool = False,  # 🆕 HTTP/2 cookie splitting

    # Browser Impersonation
    impersonate: str | None = None,  # chrome_141, safari_18, etc.
    impersonate_os: str | None = None,  # windows, macos, linux, etc.

    # Network Settings
    proxy: str | None = None,
    timeout: float = 30,
    verify: bool = True,
    ca_cert_file: str | None = None,

    # HTTP Configuration
    http2_only: bool = False,
    https_only: bool = False,
    follow_redirects: bool = True,
    max_redirects: int = 20,
    referer: bool = True,

    # Performance Optimization
    pool_idle_timeout: float | None = None,
    pool_max_idle_per_host: int | None = None,
    tcp_nodelay: bool | None = None,
    tcp_keepalive: float | None = None,

    # Retry Mechanism
    retry_count: int | None = None,
    retry_backoff: float | None = None,

    # Query Parameters
    params: dict[str, str] | None = None,
)
```

</details>

<details>
<summary><b>Request Methods</b></summary>

```python
# HTTP Methods
client.get(url, **kwargs)
client.post(url, **kwargs)
client.put(url, **kwargs)
client.patch(url, **kwargs)
client.delete(url, **kwargs)
client.head(url, **kwargs)
client.options(url, **kwargs)

# Common Parameters
params: dict[str, str] | None = None,
headers: dict[str, str] | None = None,
ordered_headers: dict[str, str] | None = None,  # 🆕
cookies: dict[str, str] | None = None,
auth: tuple[str, str | None] | None = None,
auth_bearer: str | None = None,
timeout: float | None = None,

# POST/PUT/PATCH Specific
content: bytes | None = None,
data: dict[str, Any] | None = None,
json: Any | None = None,
files: dict[str, str] | None = None,
```

</details>

<details>
<summary><b>Response Object</b></summary>

```python
response.status_code        # HTTP status code
response.headers            # Response headers
response.cookies            # Response cookies
response.url                # Final URL (after redirects)
response.encoding           # Content encoding

# Body Access
response.text               # Text content
response.content            # Binary content
response.json()             # Parse JSON
response.stream()           # Stream response body

# HTML Conversion
response.text_markdown      # HTML → Markdown
response.text_plain         # HTML → Plain text
response.text_rich          # HTML → Rich text
```

</details>

<details>
<summary><b>Supported Browsers</b></summary>

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

#### OS Support
`windows`, `macos`, `linux`, `android`, `ios`

</details>

---

## 💡 Examples

### Example 1: Web Scraping with Anti-Bot Bypass

```python
import never_primp as primp

# Perfect browser simulation
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
    retry_count=3,
)

response = client.get("https://difficult-site.com")
print(response.status_code)
```

### Example 2: API Integration with Authentication

```python
client = primp.Client(
    headers={
        "Content-Type": "application/json",
        "X-API-Version": "v1",
    },
    auth_bearer="your-api-token",
    timeout=30,
)

# GET request
data = client.get("https://api.example.com/users").json()

# POST request
response = client.post(
    "https://api.example.com/users",
    json={"name": "John", "email": "john@example.com"}
)
```

### Example 3: File Upload

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

### Example 4: Session Management

```python
# Automatic cookie persistence
client = primp.Client(cookie_store=True)

# Login
client.post(
    "https://example.com/login",
    data={"username": "user", "password": "pass"}
)

# Subsequent requests include session cookies
profile = client.get("https://example.com/profile")
```

### Example 5: Proxy Usage

```python
# SOCKS5 proxy
client = primp.Client(proxy="socks5://127.0.0.1:1080")

# HTTP proxy with authentication
client = primp.Client(proxy="http://user:pass@proxy.example.com:8080")

# Environment variable
import os
os.environ['PRIMP_PROXY'] = 'http://127.0.0.1:8080'
```

### Example 6: Async Concurrent Requests

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

### Example 7: Streaming Large Files

```python
client = primp.Client()

response = client.get("https://example.com/large-file.zip")

with open("output.zip", "wb") as f:
    for chunk in response.stream():
        f.write(chunk)
```

---

## 🎯 Use Cases

### ✅ Perfect For

- **Web Scraping**: Bypass anti-bot systems (Cloudflare, Akamai, PerimeterX)
- **API Testing**: High-performance API client with retries
- **Data Collection**: Concurrent requests with connection pooling
- **Security Research**: TLS fingerprint analysis and testing
- **Browser Automation Alternative**: Lighter than Selenium/Playwright

### ⚠️ Not Suitable For

- **JavaScript Rendering**: Use Playwright/Selenium for dynamic content
- **Browser Automation**: No DOM manipulation or JavaScript execution
- **Visual Testing**: No screenshot or rendering capabilities

---

## 🔬 Benchmarks

### Sequential Requests (Connection Reuse)

| Library | Time (10 requests) | Relative Speed |
|---------|-------------------|----------------|
| **never_primp** | 1.24s | **1.00x** (baseline) |
| httpx | 1.89s | 0.66x slower |
| requests | 3.05s | 0.41x slower |

### Concurrent Requests (AsyncClient)

| Library | Time (100 requests) | Relative Speed |
|---------|---------------------|----------------|
| **never_primp** | 2.15s | **1.00x** (baseline) |
| httpx | 2.83s | 0.76x slower |
| aiohttp | 2.45s | 0.88x slower |

*Benchmarks run on: Python 3.11, Ubuntu 22.04, AMD Ryzen 9 5900X*

---

## 🛠️ Development

### Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/never-primp.git
cd never-primp

# Create virtual environment
python -m venv venv
source venv/bin/activate  # Linux/macOS
# or
venv\Scripts\activate  # Windows

# Install maturin (Rust-Python build tool)
pip install maturin

# Build and install in development mode
maturin develop --release

# Run examples
python examples/example_ordered_headers.py
```

### Project Structure

```
never-primp/
├── src/
│   ├── lib.rs              # Main Rust implementation
│   ├── traits.rs           # Header conversion traits
│   ├── response.rs         # Response handling
│   ├── impersonate.rs      # Browser impersonation
│   └── utils.rs            # Certificate utilities
├── never_primp/
│   ├── __init__.py         # Python API wrapper
│   └── never_primp.pyi     # Type hints
├── examples/
│   ├── example_ordered_headers.py
│   └── example_split_cookies.py
├── Cargo.toml              # Rust dependencies
└── pyproject.toml          # Python package config
```

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

1. Follow Rust best practices for src/ files
2. Maintain Python 3.8+ compatibility
3. Add tests for new features
4. Update documentation

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## ⚠️ Disclaimer

This tool is intended for **educational purposes** and **legitimate use cases** only, such as:
- Testing your own applications
- Academic research
- Security auditing (with permission)
- Data collection from public APIs

**Important**:
- Respect websites' `robots.txt` and Terms of Service
- Do not use for malicious purposes or unauthorized access
- Be mindful of rate limiting and server resources
- The authors are not responsible for misuse of this tool

Use responsibly and ethically. 🙏

---

## 🙏 Acknowledgments

Built with:
- [wreq](https://github.com/0x676e67/wreq) - Rust HTTP client with browser impersonation
- [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
- [tokio](https://tokio.rs/) - Async runtime for Rust

Inspired by:
- [curl-impersonate](https://github.com/lwthiker/curl-impersonate)
- [httpx](https://github.com/encode/httpx)
- [requests](https://github.com/psf/requests)

---

## 📞 Support

- 📖 [Documentation](ORDERED_HEADERS.md)
- 🐛 [Issue Tracker](https://github.com/yourusername/never-primp/issues)
- 💬 [Discussions](https://github.com/yourusername/never-primp/discussions)

---

<div align="center">

**Made with ❤️ and ⚙️ Rust**

If you find this project helpful, please consider giving it a ⭐!

</div>
