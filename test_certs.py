#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Test certificate verification with system native certs"""

import primp

def test_https_request():
    """Test HTTPS request with certificate verification"""
    print("Testing HTTPS request with system native certificates...")

    try:
        client = primp.Client(verify=True)
        response = client.get("https://www.google.com")

        print(f"Status Code: {response.status_code}")
        print(f"URL: {response.url}")
        print("Success! Certificate verification is working with system native certs.")
        return True

    except Exception as e:
        print(f"Error: {e}")
        print("Certificate verification failed!")
        return False

def test_another_site():
    """Test another HTTPS site"""
    print("\nTesting another HTTPS site (https://github.com)...")

    try:
        client = primp.Client(verify=True)
        response = client.get("https://github.com")

        print(f"Status Code: {response.status_code}")
        print(f"URL: {response.url}")
        print("Success! Certificate verification is working.")
        return True

    except Exception as e:
        print(f"Error: {e}")
        return False

if __name__ == "__main__":
    print("=" * 60)
    print("Testing primp with rustls-native-certs (system certificates)")
    print("=" * 60)

    test1 = test_https_request()
    test2 = test_another_site()

    print("\n" + "=" * 60)
    if test1 and test2:
        print("All tests passed! Certificate verification is working correctly.")
    else:
        print("Some tests failed. Please check the error messages above.")
    print("=" * 60)
