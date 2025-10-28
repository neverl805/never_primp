import time
import statistics
import asyncio
# import aiohttp
from curl_cffi import requests as curl_requests
import requests_go
import primp


URL = "https://www.baidu.com"   # 你可以改成任意测试接口
NUM_REQUESTS = 1                # 每个库测试次数

def benchmark_sync_lib(name, func):
    times = []
    for _ in range(NUM_REQUESTS):
        start = time.perf_counter()
        func(URL)
        times.append(time.perf_counter() - start)
    print(f"{name:<12} 平均: {statistics.mean(times)*1000:.2f} ms | 最快: {min(times)*1000:.2f} ms | 最慢: {max(times)*1000:.2f} ms")


# --- curl_cffi ---
def test_curl(url):
    curl_requests.get(url)


# --- requests-go ---
def test_requests_go(url):
    requests_go.get(url)


# --- primp ---
def test_primp(url):
    primp.get(url)


# ----------------- 异步部分 (可选) -----------------
# 如果想同时比较并发性能，可以加上以下异步部分
async def benchmark_concurrent(name, func, concurrency=10):
    async def _worker():
        start = time.perf_counter()
        await func(URL)
        return time.perf_counter() - start

    start_all = time.perf_counter()
    times = await asyncio.gather(*[_worker() for _ in range(NUM_REQUESTS)])
    print(f"{name:<12} (并发) 平均: {statistics.mean(times)*1000:.2f} ms | 最快: {min(times)*1000:.2f} ms | 最慢: {max(times)*1000:.2f} ms | 总耗时: {(time.perf_counter()-start_all):.2f}s")


# 示例：用 aiohttp 测试并发性能基线
# async def test_aiohttp(url):
#     async with aiohttp.ClientSession() as session:
#         async with session.get(url) as resp:
#             await resp.text()


if __name__ == "__main__":
    print(f"目标 URL: {URL}")
    print(f"每个库请求 {NUM_REQUESTS} 次\n")

    # --- 顺序测试 ---
    benchmark_sync_lib("curl_cffi", test_curl)
    benchmark_sync_lib("requests-go", test_requests_go)
    benchmark_sync_lib("primp", test_primp)

    # --- 异步并发测试 (可选) ---
    # print("\n=== 并发性能对比 (async, baseline) ===")
    # asyncio.run(benchmark_concurrent("aiohttp", test_aiohttp))
