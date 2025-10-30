"""
示例：使用 ordered_headers 进行请求头顺序控制

这个示例展示了如何使用 ordered_headers 参数来精确控制 HTTP 请求头的顺序，
这对于绕过某些反爬虫检测非常有用。
"""

from never_primp import Client

# 示例 1: 使用 ordered_headers 创建客户端
# Python 3.7+ 的 dict 会保持插入顺序
ordered_headers = {
    "sec-ch-ua": "\"Google Chrome\";v=\"141\", \"Not?A_Brand\";v=\"8\", \"Chromium\";v=\"141\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"Windows\"",
    "upgrade-insecure-requests": "1",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
    "sec-fetch-site": "none",
    "sec-fetch-mode": "navigate",
    "sec-fetch-user": "?1",
    "sec-fetch-dest": "document",
    "Accept-Encoding": "gzip, deflate, br, zstd",
    "accept-language": "en",
    "priority": "u=0, i"
}

cookies = {
    "JSESSIONID": "025ACF32B7C98E86FBB5775FF653D842.ibe-trprv9or-5b4f9f6b5d-2k2fh",
    "_ga": "GA1.1.1369884957.1761724820",
    "acw_tc": "ac11000117617248254173247e0060459e5e269a99c888e2da2fda3d18ddb9",
    "_c_WBKFRo": "zaBnVZgRL4QicL02hnQSjz0Id8m4DBmw8klOmnuN",
    "_nb_ioWEgULi": "",
    # "acw_sc__v3": "6901c9a928391b4cc75e10e21e60c5a2f4aa2a21",
    "INGRESSCOOKIE": "1761724843.087.10302.437196|8cc0f3ff99b32de79eac79ca394f7c68",
    # "ROUTEID": ".ibe-prod14",
    # "hkaRegionAndLang": "zh_CN",
    # "_ga_ESNW6S2LPB": "GS2.1.s1761724820$o1$g1$t1761724848$j32$l0$h0",
    # "ssxmod_itna": "1-Wqmx9D0iwxu7G0D2DRxbDpxQqYK7QKDCDl4BtQtGgDYq7=GFYDCErF4GIcroGkQSoCdnXKbQjk5D/mQ7eKDU=SpieMQ0q2DIbFbG5PoMAiD2beh3dE2gmAqpmyGmE5v/CSKxTbExXgur4GLDY6vRkeRKxGGD0oDt4DIDAYDDxDWDYEvDGtQDG=D7hb=MnTdxi3DbhbDf4DmDGY31eDgGDDBDD6xZYP3gxLdrDDli07eU2DnFP9xEPqN9jx7pi43x0UWDBLxnKx3X1MUriW7vUSWTRrDzk1DtuTQKZoNmgbbVQaeXoEGKE34i4gAKoe43CqelGkQ4qCwPAPe0wKQ_KixjGDqQK6Gmfn2rDDA/vYLR=QRY7yM_ylfyYY2dBbO_oGSuxR_R/4bS2NBriK_8iBYKDPAiqgOxeoxAGDD",
    # "ssxmod_itna2": "1-Wqmx9D0iwxu7G0D2DRxbDpxQqYK7QKDCDl4BtQtGgDYq7=GFYDCErF4GIcroGkQSoCdnXKbQjkeDAKh7eTvNk9WRtsDPblInArPD"
}

client = Client(ordered_headers=ordered_headers,impersonate='chrome_140',impersonate_os='windows',verify=False)
client.proxy = 'http://127.0.0.1:9000'
print(f"  请求头数量: {len(ordered_headers)}")
print(f"  请求头顺序: {list(ordered_headers.keys())}")


response = client.get(
    cookies=cookies,
    url = "https://new.hongkongairlines.com/hxair/ibe/deeplink/ancillary.do"
)
print(f"✓ 请求成功: {response.status_code}")





