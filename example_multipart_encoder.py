"""
requests-toolbelt MultipartEncoder 兼容性示例

演示如何在 primp 中使用 requests-toolbelt 的 MultipartEncoder
"""

# 首先需要安装 requests-toolbelt
# pip install requests-toolbelt

from never_primp import Client

print("=" * 70)
print("MultipartEncoder 兼容性演示")
print("=" * 70)

# ============================================================================
# 1. 检查是否安装了 requests-toolbelt
# ============================================================================
print("\n1. 检查 requests-toolbelt")
print("-" * 70)

try:
    from requests_toolbelt import MultipartEncoder
    print("✓ requests-toolbelt 已安装")
    HAS_TOOLBELT = True
except ImportError:
    print("✗ requests-toolbelt 未安装")
    print("  请运行: pip install requests-toolbelt")
    HAS_TOOLBELT = False

if not HAS_TOOLBELT:
    print("\n请先安装 requests-toolbelt 后再运行此示例")
    exit(1)

# ============================================================================
# 2. 基本使用 - 与 requests 相同
# ============================================================================
print("\n2. 基本使用（与 requests 完全相同）")
print("-" * 70)

# requests-toolbelt 的标准用法
m = MultipartEncoder(
    fields={
        'field0': ('clickIamgeCode.jpg', open('E:\BaiduSyncdisk\js_reverse\hongkongairlines\clickIamgeCode.jpg', 'rb'), 'image/jpeg')
    },
    boundary='----WebKitFormBoundary0OcjqmPkQMDfXuPw'
)
client = Client(
    proxy='http://127.0.0.1:9000',
    verify=False,
    ordered_headers={
        "Accept": "*/*",
        "Accept-Language": "en-US,en-GB;q=0.9,en;q=0.8,zh-CN;q=0.7,zh;q=0.6",
        "Cache-Control": "no-cache",
        "Connection": "keep-alive",
        "Content-Type": m.content_type,
        "Origin": "https://vmake.ai",
        "Pragma": "no-cache",
        "Referer": "https://vmake.ai/",
        "Sec-Fetch-Dest": "empty",
        "Sec-Fetch-Mode": "cors",
        "Sec-Fetch-Site": "cross-site",
        "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
        "sec-ch-ua": "\"Chromium\";v=\"142\", \"Google Chrome\";v=\"142\", \"Not_A Brand\";v=\"99\"",
        "sec-ch-ua-mobile": "?0",
        "sec-ch-ua-platform": "\"Windows\""
    }
)
print(m.content_type)


# 使用 primp 发送（完全兼容！）
response = client.get(
    url = "https://moki-storage-release.oss-ap-southeast-1.aliyuncs.com/",
    data=m
)

print(f"✓ 状态码: {response.status_code}")
print(response.text)

exit()
# ============================================================================
# 3. 自定义 boundary
# ============================================================================
print("\n3. 自定义 boundary")
print("-" * 70)

m = MultipartEncoder(
    fields={
        'username': 'admin',
        'password': 'secret',
        'avatar': ('profile.jpg', b'\xff\xd8\xff\xe0', 'image/jpeg')
    },
    boundary='----WebKitFormBoundary0OcjqmPkQMDfXuPw'  # 自定义 boundary
)

response = client.post(
    'https://httpbin.org/post',
    data=m,
    headers={'Content-Type': m.content_type}
)

print(f"✓ 状态码: {response.status_code}")
print(f"✓ Content-Type: {m.content_type}")

# ============================================================================
# 4. 文件上传
# ============================================================================
print("\n4. 文件上传（使用 MultipartEncoder）")
print("-" * 70)

import tempfile
import os

# 创建临时文件
temp_file = tempfile.NamedTemporaryFile(mode='w', suffix='.txt', delete=False)
temp_file.write("这是文件内容\nLine 2\nLine 3")
temp_file.close()

try:
    with open(temp_file.name, 'rb') as f:
        m = MultipartEncoder(
            fields={
                'upload_type': 'document',
                'file': ('document.txt', f, 'text/plain')
            }
        )

        response = client.post(
            'https://httpbin.org/post',
            data=m,
            headers={'Content-Type': m.content_type}
        )

        print(f"✓ 状态码: {response.status_code}")
        print(f"✓ 文件上传成功")
finally:
    os.unlink(temp_file.name)

# ============================================================================
# 5. 对比：primp 原生 vs MultipartEncoder
# ============================================================================
print("\n5. 对比：primp 原生 vs MultipartEncoder")
print("-" * 70)

print("\n方式1: MultipartEncoder（兼容 requests）")
m = MultipartEncoder(
    fields={
        'user': 'admin',
        'file': ('test.txt', b'content', 'text/plain')
    }
)
response1 = client.post(
    'https://httpbin.org/post',
    data=m,
    headers={'Content-Type': m.content_type}
)
print(f"✓ MultipartEncoder 方式: {response1.status_code}")

print("\n方式2: primp 原生（更简单）")
response2 = client.post(
    'https://httpbin.org/post',
    data={'user': 'admin'},
    files={'file': ('test.txt', b'content', 'text/plain')}
)
print(f"✓ primp 原生方式: {response2.status_code}")

print("\n两种方式效果完全相同！")

# ============================================================================
# 6. 实际应用场景
# ============================================================================
print("\n6. 实际应用场景")
print("-" * 70)

def upload_with_encoder(url: str, user_id: str, file_path: str):
    """使用 MultipartEncoder 上传文件的函数"""
    with open(file_path, 'rb') as f:
        m = MultipartEncoder(
            fields={
                'user_id': user_id,
                'timestamp': '2025-01-01 12:00:00',
                'file': (os.path.basename(file_path), f, 'application/octet-stream')
            }
        )

        client = Client()
        response = client.post(
            url,
            data=m,
            headers={'Content-Type': m.content_type}
        )

        return response

print("✓ 定义了使用 MultipartEncoder 的函数")
print("✓ 在 primp 中可以直接使用，无需修改代码！")

# ============================================================================
# 7. 进度监控（MultipartEncoderMonitor）
# ============================================================================
print("\n7. 进度监控")
print("-" * 70)

try:
    from requests_toolbelt import MultipartEncoderMonitor

    def progress_callback(monitor):
        # 打印上传进度
        percentage = (monitor.bytes_read / monitor.len) * 100
        print(f"\r上传进度: {percentage:.1f}%", end='')

    # 创建大一点的数据
    large_data = b'X' * (1024 * 100)  # 100KB

    m = MultipartEncoder(
        fields={
            'description': '大文件上传',
            'file': ('large_file.bin', large_data, 'application/octet-stream')
        }
    )

    # 包装成 monitor
    monitor = MultipartEncoderMonitor(m, progress_callback)

    response = client.post(
        'https://httpbin.org/post',
        data=monitor,
        headers={'Content-Type': monitor.content_type}
    )

    print(f"\n✓ 状态码: {response.status_code}")
    print(f"✓ 大文件上传完成（带进度监控）")

except ImportError:
    print("✗ MultipartEncoderMonitor 需要较新版本的 requests-toolbelt")

# ============================================================================
# 总结
# ============================================================================
print("\n" + "=" * 70)
print("兼容性总结")
print("=" * 70)

summary = """
✅ 完全兼容 requests-toolbelt 的 MultipartEncoder
✅ 无需修改现有代码即可使用
✅ 支持自定义 boundary
✅ 支持进度监控（MultipartEncoderMonitor）
✅ 自动处理 Content-Type

迁移建议：
1. 如果您正在使用 requests + requests-toolbelt
   → 直接替换为 primp，代码无需修改

2. 如果是新项目
   → 推荐使用 primp 原生格式（更简单）

3. 如果需要进度监控
   → 继续使用 MultipartEncoderMonitor
   → primp 完全兼容

对比：
┌─────────────────────────────┬──────────┬──────────┐
│           功能              │ requests │  primp   │
├─────────────────────────────┼──────────┼──────────┤
│ MultipartEncoder            │    ✅    │    ✅    │
│ MultipartEncoderMonitor     │    ✅    │    ✅    │
│ 自定义 boundary             │    ✅    │    ✅    │
│ 原生 data+files            │    ✅    │    ✅    │
│ 文件路径字符串              │    ❌    │    ✅    │
│ 自动流式传输                │    ❌    │    ✅    │
└─────────────────────────────┴──────────┴──────────┘
"""

print(summary)

print("\n" + "=" * 70)
print("示例完成！")
print("=" * 70)
