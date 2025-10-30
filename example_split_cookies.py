"""
示例：使用 split_cookies 控制 Cookie 发送方式

演示如何使用 split_cookies 参数来模拟浏览器的 HTTP/2 cookie 行为
"""

from never_primp import Client


# 准备测试 cookies
test_cookies = {
    "buvid3": "7A413405-3B2F-7DA8-4773-FACA58DC3B5A23886infoc",
    "b_nut": "1761796023",
    "b_lsid": "83F10657A_19A3339E8C7",
    "session_id": "abc123def456",
}

bilibili_client = Client(

    split_cookies=False,  # 使用 HTTP/2 风格
    ordered_headers={
        "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "sec-ch-ua": '"Google Chrome";v="141", "Not?A_Brand";v="8"',
        "accept": "*/*",
        "origin": "https://www.bilibili.com",
        "sec-fetch-site": "same-site",
        "sec-fetch-mode": "cors",
        "sec-fetch-dest": "empty",
        "referer": "https://www.bilibili.com/",
    }
)
bilibili_client.proxy = 'http://127.0.0.1:7890'
response = bilibili_client.get('https://www.bilibili.com',cookies=test_cookies)
print(f"✓ 请求成功: {response.status_code}")


