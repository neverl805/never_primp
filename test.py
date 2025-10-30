from never_primp import Client

headers = {
    "accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7",
    "accept-language": "en,zh-CN;q=0.9,zh;q=0.8,zh-HK;q=0.7",
    "cache-control": "no-cache",
    "pragma": "no-cache",
    "priority": "u=0, i",
    "referer": "https://hisky.aero/",
    "sec-ch-ua": "\"Google Chrome\";v=\"141\", \"Not?A_Brand\";v=\"8\", \"Chromium\";v=\"141\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"Windows\"",
    "sec-fetch-dest": "document",
    "sec-fetch-mode": "navigate",
    "sec-fetch-site": "same-site",
    "sec-fetch-user": "?1",
    "upgrade-insecure-requests": "1",
    "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36"
}
cookies = {
    "_gcl_au": "1.1.223070972.1761814157",
    "cf_clearance": "sbUF9oZS4pm8XVVNE.dnxxtZA.ZaK_IUE1lAmJTQfME-1761814156-1.2.1.1-0j8chJiO99fUkkj9ScKLe3_NzwwYuNoB_8Y9nZf5GTAa0Fn0l8kcDxngl1aISqK7ZDiJ.YN1cfipphzCIn46LkoBNjVryWuORy4m_tEhJQDlaptDTEoVnYoe4MmrLRcTIFia4sDvJ5CdSiTqewQsawG51.w_thq8MRZ9Z2Obf7SoV6MNC2eyRbCm96orSApfuK2tURiL_0K9EeNfES92Ss8kDoKZXBane0dwO_5vdMY",
    "_gid": "GA1.2.339976247.1761814157",
    "_gat_UA-155319482-1": "1",
    "_fbp": "fb.1.1761814157993.834210106761768202",
    "ASP.NET_SessionId": "0eme0emlicujjzldxxilfrrh",
    "_ga": "GA1.2.1350469137.1761814157",
    "_ga_0KDTF1TCBJ": "GS2.1.s1761814157$o1$g1$t1761814185$j32$l0$h0"
}
url = "https://booking.hisky.aero/VARS/Public/deeplink.aspx"
params = {
    "TripType": "roundtrip",
    "UserLanguage": "ro",
    "Adult": "1",
    "Child": "0",
    "InfantLap": "0",
    "UserCurrency": "USD",
    "Origin1": "OTP",
    "Destination2": "OTP",
    "Destination1": "TSR",
    "Origin2": "TSR",
    "DepartureDate1": "31.10.2025",
    "DepartureDate2": "31.10.2025",
    "flightvoucher": ""
}
session = Client(proxy='http://127.0.0.1:7890')

response = session.get(url, headers=headers, cookies=cookies, params=params)

print(response.url)
print(response.headers)
print(response.cookies)
print(session.get_all_cookies())