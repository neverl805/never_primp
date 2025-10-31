# Ordered Headers - 有序请求头

## 为什么需要？

部分网站的反爬虫系统会检测 HTTP 请求头的**顺序**，普通 HTTP 客户端无法保证顺序，导致请求被识别为机器人。

## 头部顺序规则

使用 `ordered_headers` 时，NEVER_PRIMP 会按照以下严格顺序构建请求头：

```
1. Host (如果存在于 ordered_headers 中)
2. Content-Length (POST/PUT/PATCH 且有请求体时自动添加)
3. Content-Type (如果自动计算，例如 json 参数)
4. 其他自定义头部 (按 ordered_headers 中的顺序，排除 priority 和 cookie)
5. cookie (倒数第二位，如果有 cookie)
6. priority (最后一位，如果存在)
```

**示例**：
```python
client = Client(
    ordered_headers={
        "Host": "api.example.com",
        "User-Agent": "Mozilla/5.0...",
        "Accept": "application/json",
        "Accept-Encoding": "gzip",
        "priority": "u=1, i"  # 会被移到最后
    }
)

# POST 请求时，实际发送的头部顺序：
# Host: api.example.com
# Content-Length: 123
# Content-Type: application/json
# User-Agent: Mozilla/5.0...
# Accept: application/json
# Accept-Encoding: gzip
# priority: u=1, i
```

### Cookie 顺序说明

- **默认 (`split_cookies=False`)**: cookie 作为单个头部在倒数第二位
- **分割模式 (`split_cookies=True`)**: 多个 cookie 头部都在 priority 之前

```python
# split_cookies=False
# ...其他头部...
# cookie: session=abc; token=xyz
# priority: u=1, i

# split_cookies=True
# ...其他头部...
# cookie: session=abc
# cookie: token=xyz
# priority: u=1, i
```

📖 更多信息请参考 [SPLIT_COOKIES.md](SPLIT_COOKIES.md)

## 快速使用

### 客户端级别

```python
from never_primp import Client

client = Client(
    ordered_headers={
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "accept-language": "en-US,en;q=0.9",
        "accept-encoding": "gzip, deflate, br",
        "sec-fetch-dest": "document",
        "sec-fetch-mode": "navigate",
    }
)

response = client.get("https://example.com")
```

### 单次请求

```python
client = Client()

response = client.get(
    "https://example.com",
    ordered_headers={
        "authorization": "Bearer token",
        "accept": "application/json",
    }
)
```

### 动态修改

```python
# 完全替换
client.ordered_headers = {...}

# 增量更新（保持原有顺序，更新值）
client.ordered_headers_update({"referer": "https://google.com"})

# 获取当前设置
current = client.ordered_headers
```

## 实战示例

### Chrome 浏览器完整模拟

```python
client = Client(
    impersonate="chrome_141",
    ordered_headers={
        "sec-ch-ua": '"Chromium";v="141", "Not?A_Brand";v="8"',
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": '"Windows"',
        "upgrade-insecure-requests": "1",
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        "sec-fetch-site": "none",
        "sec-fetch-mode": "navigate",
        "sec-fetch-user": "?1",
        "sec-fetch-dest": "document",
        "accept-encoding": "gzip, deflate, br, zstd",
        "accept-language": "en-US,en;q=0.9",
        "priority": "u=0, i",  # 自动移到最后
    },
    http2_only=True,
)

# 注意：Host 和 Content-Length 会自动添加到正确位置
# priority 会自动移到最后一位
```

**实际发送的头部顺序**（POST 请求）：
```
Host: example.com
Content-Length: 123
Content-Type: application/json
sec-ch-ua: "Chromium";v="141", "Not?A_Brand";v="8"
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "Windows"
upgrade-insecure-requests: 1
user-agent: Mozilla/5.0...
accept: text/html,application/xhtml+xml...
sec-fetch-site: none
sec-fetch-mode: navigate
sec-fetch-user: ?1
sec-fetch-dest: document
accept-encoding: gzip, deflate, br, zstd
accept-language: en-US,en;q=0.9
priority: u=0, i
```

### 获取真实浏览器的请求头顺序

**方法 1**: 使用 Chrome DevTools
1. 打开 DevTools (F12)
2. Network 标签
3. 访问目标网站
4. 查看 Request Headers，按显示顺序记录

**方法 2**: 使用抓包工具（推荐）
- Reqable
- Charles Proxy
- mitmproxy

查看 **Raw Request**，复制请求头顺序。

## 技术细节

### 与 headers 的区别

| 特性 | `headers` | `ordered_headers` |
|------|-----------|------------------|
| 底层实现 | `HeaderMap` | `OrigHeaderMap` |
| 顺序保证 | ❌ 大致保序 | ✅ 严格保序 |
| 大小写保持 | ❌ 标准化 | ✅ 原始形式 |
| 性能 | 快 | 稍慢（<5%）|
| 适用场景 | 普通请求 | 反爬虫绕过 |

### 优先级

```python
client = Client(
    headers={"user-agent": "ignored"},
    ordered_headers={"user-agent": "used"}  # 优先使用
)
```

`ordered_headers` > `headers`

## 常见问题

**Q: 如何知道网站是否检测请求头顺序？**

A: 对比测试法
1. 使用 `headers` → 被拒绝
2. 使用 `ordered_headers`（模拟浏览器顺序）→ 成功
3. 说明网站检测了顺序

**Q: Python dict 保持顺序吗？**

A: Python 3.7+ 的 dict **保持插入顺序**，可以直接使用。

**Q: 与 impersonate 配合使用？**

A: `impersonate` 会覆盖自定义头部。如需精确控制，不要使用 `impersonate`，手动配置 `ordered_headers`。

**Q: 性能影响？**

A: <5%，实际应用中可忽略。

## 最佳实践

1. **默认不使用**：仅在需要时启用（被检测时）
2. **复制真实浏览器**：使用抓包工具获取真实顺序
3. **配合其他功能**：`ordered_headers` + `split_cookies` + `http2_only` = 完美模拟
4. **注意大小写**：保持与浏览器一致（通常首字母大写或全小写）
5. **不要手动添加 Host 和 Content-Length**：它们会自动添加到正确位置
6. **priority 自动排序**：无需担心位置，会自动移到最后
7. **cookie 自动排序**：无需在 ordered_headers 中包含 cookie，使用 `cookies` 参数即可

## 调试技巧

### 验证顺序是否正确

```python
# 使用 httpbin.org
response = client.get("https://httpbin.org/headers")
print(response.json()["headers"])

# 使用代理抓包
client = Client(
    proxy="http://127.0.0.1:8888",  # Reqable/Charles
    verify=False,
    ordered_headers={...}
)
```

在抓包工具中查看 **Request Headers**，确认顺序。

---

**总结**：`ordered_headers` 是反爬虫的利器，在需要精确模拟浏览器时使用。
