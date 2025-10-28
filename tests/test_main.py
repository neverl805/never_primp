import primp

# Impersonate
client = primp.Client(impersonate="chrome_141", impersonate_os="windows")
resp = client.get("https://tls.peet.ws/api/all")
print(resp.text)

