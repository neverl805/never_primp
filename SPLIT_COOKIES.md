# Split Cookies - Cookie 分割发送

## 问题背景

真实浏览器在 **HTTP/2** 中会将 Cookie 分割成多个独立的头部发送：

```http
cookie: session_id=abc123
cookie: user_token=xyz789
cookie: preference=dark
priority: u=1, i
```

而不是合并成一个：

```http
Cookie: session_id=abc123; user_token=xyz789; preference=dark
priority: u=1, i
```

部分反爬虫系统会检测这个细节。

## 重要：Cookie 与 Priority 的顺序

无论使用何种模式，**priority 头部始终在最后一位，cookie 头部在 priority 之前**。

### 顺序规则

```
...其他头部...
cookie: xxx (单个或多个，取决于 split_cookies)
priority: xxx (如果存在，始终最后)
```

**示例对比**：

```python
# split_cookies=False (默认)
# Host: example.com
# Content-Type: application/json
# User-Agent: Mozilla/5.0...
# ...其他头部...
# cookie: session=abc; token=xyz; lang=en
# priority: u=1, i

# split_cookies=True
# Host: example.com
# Content-Type: application/json
# User-Agent: Mozilla/5.0...
# ...其他头部...
# cookie: session=abc
# cookie: token=xyz
# cookie: lang=en
# priority: u=1, i
```

📖 更多顺序规则请参考 [ORDERED_HEADERS.md](ORDERED_HEADERS.md)

## 快速使用

### 启用分割模式

```python
from never_primp import Client

# 客户端级别
client = Client(
    split_cookies=True,  # 启用 Cookie 分割
    http2_only=True,     # 建议配合 HTTP/2
)

response = client.get(
    "https://example.com",
    cookies={"session": "abc", "user_id": "123"}
)
```

### 发送效果

**split_cookies=False**（默认）:
```http
Cookie: session=abc; user_id=123
```

**split_cookies=True**:
```http
cookie: session=abc
cookie: user_id=123
```

### 动态切换

```python
client = Client()

# 切换到分割模式
client.split_cookies = True

# 切换回合并模式
client.split_cookies = False
```

## HTTP 标准说明

### HTTP/1.1 (RFC 6265)
- ✅ 必须合并：单个 `Cookie` 头部
- ❌ 不允许多个 `Cookie` 头部

### HTTP/2 (RFC 9113)
- ✅ 允许分割：多个 `cookie` 头部（小写）
- ✅ Chrome/Firefox 的实际行为
- ✅ 目的：更好的 HPACK 压缩

## 实战示例

### 完美的浏览器模拟

```python
client = Client(
    # Cookie 分割
    split_cookies=True,

    # HTTP/2
    http2_only=True,

    # 有序请求头
    ordered_headers={
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "sec-ch-ua": '"Chromium";v="141", "Not?A_Brand";v="8"',
        "accept": "*/*",
        "sec-fetch-site": "same-site",
        "sec-fetch-mode": "cors",
        "sec-fetch-dest": "empty",
        "referer": "https://www.example.com/",
        "priority": "u=1, i",  # 自动移到最后
    },
)

# Cookie 会被分割发送，且在 priority 之前
response = client.post(
    "https://api.example.com/data",
    cookies={
        "buvid3": "7A413405-3B2F-7DA8-4773-FACA58DC3B5A23886infoc",
        "b_nut": "1761796023",
        "b_lsid": "83F10657A_19A3339E8C7",
    }
)
```

发送的请求头（实际顺序）：
```http
Host: api.example.com
Content-Length: 123
Content-Type: application/json
user-agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36
sec-ch-ua: "Chromium";v="141", "Not?A_Brand";v="8"
accept: */*
sec-fetch-site: same-site
sec-fetch-mode: cors
sec-fetch-dest: empty
referer: https://www.example.com/
cookie: buvid3=7A413405-3B2F-7DA8-4773-FACA58DC3B5A23886infoc
cookie: b_nut=1761796023
cookie: b_lsid=83F10657A_19A3339E8C7
priority: u=1, i
```

注意：
- Host 和 Content-Length 自动添加在最前面
- priority 自动移到最后一位
- cookie 在 priority 之前

### 标准 API 调用

```python
client = Client(
    split_cookies=False,  # 使用标准方式
)

response = client.post(
    "https://api.example.com/users",
    json={"name": "John"},
    cookies={"session_id": "abc123"}
)
```

## 使用建议

### 何时使用 split_cookies=True

- ✅ 目标网站使用 HTTP/2
- ✅ 需要精确模拟浏览器
- ✅ 反爬虫检测 Cookie 格式
- ✅ 与 `ordered_headers` 配合使用

### 何时使用 split_cookies=False（默认）

- ✅ 目标网站使用 HTTP/1.1
- ✅ 标准 API 调用
- ✅ 兼容性优先
- ✅ 服务器不关心 Cookie 格式

## 决策树

```
需要模拟真实浏览器？
├─ 是 → 目标使用 HTTP/2？
│      ├─ 是 → ✅ split_cookies=True + ordered_headers + http2_only=True
│      └─ 否 → ⚠️  split_cookies=False (HTTP/1.1 不支持分割)
└─ 否 → ✅ split_cookies=False (默认即可)
```

## 技术细节

### 实现原理

**合并模式** (`split_cookies=False`):
```rust
let cookie_value = cookies.join("; ");
request.header("Cookie", cookie_value);
```

**分割模式** (`split_cookies=True`):
```rust
for (name, value) in cookies {
    request.header_append("cookie", format!("{}={}", name, value));
}
```

### 服务器端处理

服务器**必须支持两种格式**（RFC 9113 要求）：
- 单个 `Cookie` 头部（HTTP/1.1）
- 多个 `cookie` 头部（HTTP/2）

服务器会自动合并多个 `cookie` 头部。

## 常见问题

**Q: 性能影响？**

A: <1%，可忽略。HTTP/2 + 分割 Cookie 可能有更好的压缩率。

**Q: HTTP/1.1 可以使用吗？**

A: 技术上可以，但**违反 RFC 6265 标准**。不推荐。

**Q: 如何验证是否生效？**

A: 使用抓包工具（Reqable/Charles）查看 **Raw Request**，确认 Cookie 格式。

**Q: 与 impersonate 配合？**

A: `impersonate` 会覆盖 `split_cookies` 设置。如需自定义，不要使用 `impersonate`。

**Q: priority 头部的顺序？**

A: **priority 始终在最后一位**，无论 split_cookies 设置如何。Cookie（单个或多个）都在 priority 之前。

**Q: 如何同时使用 ordered_headers、split_cookies 和 priority？**

A: 只需在 `ordered_headers` 中包含 priority，它会自动移到最后。Cookie 通过 `cookies` 参数传递，会自动排列在 priority 之前。

```python
client = Client(
    split_cookies=True,
    ordered_headers={
        "user-agent": "...",
        "accept": "...",
        "priority": "u=1, i",  # 会自动移到最后
    }
)

response = client.post(url, cookies={"a": "1", "b": "2"})
# 顺序: ...其他头部... → cookie: a=1 → cookie: b=2 → priority: u=1, i
```

## 调试验证

### 使用抓包工具

```python
client = Client(
    proxy="http://127.0.0.1:8888",  # Reqable/Charles 端口
    verify=False,
    split_cookies=True,
)

response = client.get(
    "https://example.com",
    cookies={"a": "1", "b": "2", "c": "3"}
)
```

在抓包工具中查看请求头：
- 分割模式：看到多个 `cookie:` 行
- 合并模式：看到单个 `Cookie:` 行

### 使用 httpbin.org

```python
response = client.get(
    "https://httpbin.org/headers",
    cookies={"test1": "value1", "test2": "value2"}
)

print(response.json()["headers"]["Cookie"])
```

注意：httpbin 会自动合并显示，需用抓包工具看真实格式。

## 配置组合建议

### 完美浏览器模拟

```python
client = Client(
    split_cookies=True,
    ordered_headers={...},
    http2_only=True,
    impersonate=None,  # 不使用，完全自定义
)
```

### 标准 API 客户端

```python
client = Client(
    split_cookies=False,  # 或省略，默认 False
    headers={...},
)
```

### 灵活方案

```python
client = Client(split_cookies=True)

# 根据目标动态调整
if is_http1_only:
    client.split_cookies = False
```

---

**总结**：`split_cookies` 用于精确模拟 HTTP/2 浏览器行为，绕过检测 Cookie 格式的反爬虫系统。
