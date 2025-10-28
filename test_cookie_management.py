#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Cookie 管理示例和工具类"""

import never_primp as primp
from http.cookies import SimpleCookie
from typing import Dict, Optional, Any


class CookieManager:
    """Cookie 管理工具类 - 用于手动管理 primp 客户端的 cookies"""

    def __init__(self):
        self.cookies: Dict[str, str] = {}

    def extract_from_response(self, response: Any):
        """从响应中提取 cookies"""
        set_cookie_headers = response.headers.get('set-cookie', '').split('\n')

        for header_value in set_cookie_headers:
            if not header_value:
                continue

            # 使用 SimpleCookie 解析
            cookie = SimpleCookie()
            cookie.load(header_value)

            for key, morsel in cookie.items():
                self.cookies[key] = morsel.value

    def get_cookies_dict(self) -> Dict[str, str]:
        """获取 cookies 字典"""
        return self.cookies.copy()

    def set_cookie(self, name: str, value: str):
        """设置单个 cookie"""
        self.cookies[name] = value

    def remove_cookie(self, name: str):
        """删除单个 cookie"""
        self.cookies.pop(name, None)

    def clear(self):
        """清空所有 cookies"""
        self.cookies.clear()

    def __repr__(self):
        return f"CookieManager({len(self.cookies)} cookies)"


def test_automatic_cookie_management():
    """测试1: 自动 Cookie 管理（默认行为）"""
    print("\n" + "="*60)
    print("测试1: 自动 Cookie 管理")
    print("="*60)

    # 创建客户端，cookie_store 默认为 True
    client = primp.Client()

    # 第一次请求 - httpbin 会设置一个 cookie
    print("\n请求1: 设置 cookie")
    response1 = client.get("https://httpbin.org/cookies/set?name=value123")
    print(f"状态: {response1.status_code}")

    # 第二次请求 - 检查 cookie 是否自动发送
    print("\n请求2: 检查 cookie 是否自动发送")
    response2 = client.get("https://httpbin.org/cookies")
    print(f"状态: {response2.status_code}")
    print(f"响应中的 cookies: {response2.text[:200]}")

    if "name" in response2.text and "value123" in response2.text:
        print("\n自动 Cookie 管理测试通过!")
        return True
    else:
        print("\n自动 Cookie 管理可能未生效")
        return False


def test_manual_cookie_management():
    """测试2: 手动 Cookie 管理"""
    print("\n" + "="*60)
    print("测试2: 手动 Cookie 管理")
    print("="*60)

    # 禁用自动 cookie 管理
    client = primp.Client(cookie_store=False)
    cookie_manager = CookieManager()

    # 第一次请求
    print("\n请求1: 获取 cookies")
    response1 = client.get("https://httpbin.org/cookies/set?session=abc123")
    print(f"状态: {response1.status_code}")

    # 提取 cookies
    cookie_manager.extract_from_response(response1)
    print(f"提取的 cookies: {cookie_manager.get_cookies_dict()}")

    # 第二次请求，手动传递 cookies
    print("\n请求2: 使用提取的 cookies")
    response2 = client.get(
        "https://httpbin.org/cookies",
        cookies=cookie_manager.get_cookies_dict()
    )
    print(f"状态: {response2.status_code}")
    print(f"响应中的 cookies: {response2.text[:200]}")

    if "session" in response2.text and "abc123" in response2.text:
        print("\n手动 Cookie 管理测试通过!")
        return True
    else:
        print("\n手动 Cookie 管理失败")
        return False


def test_cookie_manager_class():
    """测试3: CookieManager 工具类"""
    print("\n" + "="*60)
    print("测试3: CookieManager 工具类")
    print("="*60)

    manager = CookieManager()

    # 手动设置 cookies
    manager.set_cookie("user_id", "12345")
    manager.set_cookie("session_token", "abcdef")
    print(f"手动设置的 cookies: {manager.get_cookies_dict()}")

    # 使用 cookies 发送请求
    client = primp.Client(cookie_store=False)
    response = client.post(
        "https://httpbin.org/cookies",
        cookies=manager.get_cookies_dict()
    )
    print(f"状态: {response.status_code}")
    print(f"发送的 cookies: {response.text[:200]}")

    # 删除一个 cookie
    manager.remove_cookie("user_id")
    print(f"删除后的 cookies: {manager.get_cookies_dict()}")

    # 清空所有 cookies
    manager.clear()
    print(f"清空后的 cookies: {manager.get_cookies_dict()}")

    print("\nCookieManager 工具类测试完成!")
    return True


def test_session_simulation():
    """测试4: 模拟完整会话（登录 -> 访问）"""
    print("\n" + "="*60)
    print("测试4: 模拟完整会话")
    print("="*60)

    client = primp.Client(
        cookie_store=False,  # 手动管理
        headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/120.0.0.0"}
    )
    cookie_manager = CookieManager()

    # 步骤1: 模拟登录
    print("\n步骤1: 模拟登录")
    login_response = client.post(
        "https://httpbin.org/cookies/set",
        params={"auth_token": "secret123", "user": "testuser"}
    )
    print(f"登录状态: {login_response.status_code}")

    # 提取登录后的 cookies
    cookie_manager.extract_from_response(login_response)
    print(f"登录后的 cookies: {cookie_manager.get_cookies_dict()}")

    # 步骤2: 使用 session 访问受保护资源
    print("\n步骤2: 访问受保护资源")
    protected_response = client.get(
        "https://httpbin.org/cookies",
        cookies=cookie_manager.get_cookies_dict()
    )
    print(f"访问状态: {protected_response.status_code}")
    print(f"响应内容: {protected_response.text[:200]}")

    if "auth_token" in protected_response.text and "secret123" in protected_response.text:
        print("\n会话模拟测试通过!")
        return True
    else:
        print("\n会话模拟失败")
        return False


def test_persistent_cookies():
    """测试5: Cookie 持久化"""
    print("\n" + "="*60)
    print("测试5: Cookie 持久化（序列化/反序列化）")
    print("="*60)

    import json

    # 创建并设置 cookies
    manager = CookieManager()
    manager.set_cookie("persistent_id", "999")
    manager.set_cookie("preferences", "dark_mode")

    # 序列化到 JSON
    cookies_json = json.dumps(manager.get_cookies_dict())
    print(f"序列化的 cookies: {cookies_json}")

    # 模拟保存到文件/数据库
    with open("cookies.json", "w") as f:
        f.write(cookies_json)

    # 从文件读取并恢复
    with open("cookies.json", "r") as f:
        loaded_cookies = json.loads(f.read())

    new_manager = CookieManager()
    for name, value in loaded_cookies.items():
        new_manager.set_cookie(name, value)

    print(f"恢复的 cookies: {new_manager.get_cookies_dict()}")

    # 清理
    import os
    os.remove("cookies.json")

    print("\nCookie 持久化测试完成!")
    return True


if __name__ == "__main__":
    print("="*60)
    print("primp Cookie 管理测试")
    print("="*60)

    results = []

    try:
        results.append(("自动 Cookie 管理", test_automatic_cookie_management()))
    except Exception as e:
        print(f"\n自动 Cookie 管理测试失败: {e}")
        results.append(("自动 Cookie 管理", False))

    try:
        results.append(("手动 Cookie 管理", test_manual_cookie_management()))
    except Exception as e:
        print(f"\n手动 Cookie 管理测试失败: {e}")
        results.append(("手动 Cookie 管理", False))

    try:
        results.append(("CookieManager 工具类", test_cookie_manager_class()))
    except Exception as e:
        print(f"\nCookieManager 工具类测试失败: {e}")
        results.append(("CookieManager 工具类", False))

    try:
        results.append(("会话模拟", test_session_simulation()))
    except Exception as e:
        print(f"\n会话模拟测试失败: {e}")
        results.append(("会话模拟", False))

    try:
        results.append(("Cookie 持久化", test_persistent_cookies()))
    except Exception as e:
        print(f"\nCookie 持久化测试失败: {e}")
        results.append(("Cookie 持久化", False))

    # 测试总结
    print("\n" + "="*60)
    print("测试总结")
    print("="*60)
    for test_name, passed in results:
        status = "通过" if passed else "失败"
        print(f"{test_name}: {status}")

    total = len(results)
    passed_count = sum(1 for _, p in results if p)
    print(f"\n总计: {passed_count}/{total} 测试通过")
    print("="*60)

    print("\nCookie 管理要点:")
    print("  1. 简单场景: 使用 cookie_store=True (默认)")
    print("  2. 复杂场景: 使用 CookieManager 手动管理")
    print("  3. 持久化: 序列化 cookies 到文件/数据库")
    print("  4. 爬虫场景: 结合浏览器伪装 + 手动 Cookie 管理")
