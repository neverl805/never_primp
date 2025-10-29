import never_primp as primp

# Automatic cookie management (default)
url = 'https://ip.smartdaili-china.com/json'
client = primp.Client(cookie_store=True,
                      # proxy="http://127.0.0.1:7890"
                      )
client.proxy = 'http://127.0.0.1:7890'
response = client.get(url)
print(response.text)
# Cookies automatically stored and sent in next request

# Manual cookie management
cookies = client.get_cookies(url)  # Get all cookies
print(cookies)  # {'session': 'abc123'}

# Set cookies manually
client.set_cookies(url, {"auth": "token123"})
cookies = client.get_cookies(url)  # Get all cookies
print(cookies)  # {'session': 'abc123'}