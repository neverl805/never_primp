# -*- coding: utf-8 -*-
"""
测试 Content-Length 在第二位的实现
"""

from never_primp import Client


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
client = Client(proxy='http://127.0.0.1:9000',
                cookies=cookies,
                split_cookies=True,
                verify=False)

params = {
    'runtime': 'native',
    'pc_version': '1.77.8',
    'version': '1.77.8',
    'chromium_version': '135.0.7049.72',
    'real_aid': '582478',
    'aid': '582478',
}
json_data = [
    {
        'events': [
            {
                'event': 'tab_alive',
                'local_time_ms': 1761897984519,
                'params': '{"duration":60000,"tab_id":"4568064"}',
                'user_id': '1150559762528371',
            },
        ],
        'header': {
            'ab_sdk_version': '10198361,14082141',
            'app_channel': 'dbkhd_baidu_pz_pc_tuwen_zu1_title_windows',
            'app_id': 582478,
            'app_name': 'doubao',
            'app_version': '1.77.8',
            'architecture_app': 'x86_64',
            'architecture_chip': 'x86_64',
            'custom': {
                'aid': '582478',
                'chromium_version': '135.0.7049.72',
                'client_version': '1.77.8',
                'cuid': '1150559762528371',
                'current_launch_time': 1761637844638,
                'device_id_custom': '1871839391952076',
                'enable_local_homepage': 0,
                'first_launch_time': 1759977035133,
                'is_cici': False,
                'is_new': False,
                'package_name': 'doubao_client_dbkhd_baidu_pz_pc_tuwen_zu1_title_windows_v1074005_1b73_1758798841',
                'pervious_launch_time': 1761626484305,
                'pervious_shutdown_time': 1761631473688,
                'real_aid': '582478',
                'runtime': 'native',
                'samantha_version_client': '1.77.8',
                'space_type': 'personal',
                'ug_utm_source': 'dbkhd_baidu_pz_pc_tuwen_zu1_title_windows',
                'user_agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.7049.72 Safari/537.36',
                'version': '1.77.8',
                'whether_default_browser': 0,
            },
            'device_model': 'System Product Name',
            'language': 'zh-CN',
            'os_name': 'windows',
            'os_version': '10.0.26200',
            'platform': 'desktop',
            'utm_source': 'dbkhd_baidu_pz_pc_tuwen_zu1_title_windows',
        },
        'user': {
            'device_id': '1871839391952076',
            'user_id': '1150559762528371',
            'user_is_login': True,
            'user_unique_id': '1871839391952076',
        },
        'verbose': 0,
    },
]
response = client.post(
    'https://mcs.zijieapi.com/v1/list',
    params=params,
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/135.0.0.0 Safari/537.36',
        'Accept-Encoding': 'gzip, deflate, br, zstd',
        'Content-Type': 'application/json',
        'sec-fetch-site': 'none',
        'sec-fetch-mode': 'no-cors',
        'sec-fetch-dest': 'empty',
        'sec-fetch-storage-access': 'active',
        'accept-language': 'zh-CN,zh;q=0.9',
        'priority': 'u=4, i',
    },
    json=json_data
)

print(f"状态码: {response.text}")
print("\n服务器接收到的headers:")
