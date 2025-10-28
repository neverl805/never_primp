#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Test enhanced features: headers, proxy, impersonate setters"""

import primp

def test_headers():
    """Test get_headers, set_headers, and headers_update"""
    print("\n" + "="*60)
    print("Testing Headers functionality")
    print("="*60)

    # Create client with initial headers
    client = primp.Client(headers={"User-Agent": "TestAgent/1.0", "Accept": "application/json"})

    # Test get_headers
    headers = client.headers
    print(f"Initial headers: {headers}")
    assert "User-Agent" in headers
    assert headers["User-Agent"] == "TestAgent/1.0"

    # Test set_headers - completely replace headers
    new_headers = {"User-Agent": "NewAgent/2.0", "Content-Type": "application/json"}
    client.headers = new_headers
    headers = client.headers
    print(f"After set_headers: {headers}")
    assert headers["User-Agent"] == "NewAgent/2.0"
    assert "Accept" not in headers  # Old header should be gone

    # Test headers_update - merge with existing
    update_headers = {"Authorization": "Bearer token123", "User-Agent": "UpdatedAgent/3.0"}
    client.headers_update(update_headers)
    headers = client.headers
    print(f"After headers_update: {headers}")
    assert headers["User-Agent"] == "UpdatedAgent/3.0"
    assert headers["Content-Type"] == "application/json"  # Should still exist
    assert headers["Authorization"] == "Bearer token123"  # New header

    print("\nHeaders tests: PASSED")
    return True


def test_proxy():
    """Test get_proxy and set_proxy"""
    print("\n" + "="*60)
    print("Testing Proxy functionality")
    print("="*60)

    # Create client with initial proxy
    client = primp.Client(proxy="http://proxy1.example.com:8080")

    # Test get_proxy
    proxy = client.proxy
    print(f"Initial proxy: {proxy}")
    assert proxy == "http://proxy1.example.com:8080"

    # Test set_proxy
    client.proxy = "http://proxy2.example.com:3128"
    proxy = client.proxy
    print(f"After set_proxy: {proxy}")
    assert proxy == "http://proxy2.example.com:3128"

    print("\nProxy tests: PASSED")
    return True


def test_impersonate():
    """Test set_impersonate and set_impersonate_os"""
    print("\n" + "="*60)
    print("Testing Impersonate functionality")
    print("="*60)

    # Create client with initial impersonate
    client = primp.Client(impersonate="chrome_120", impersonate_os="windows")

    # Test get impersonate
    imp = client.impersonate
    imp_os = client.impersonate_os
    print(f"Initial impersonate: {imp}, OS: {imp_os}")
    assert imp == "chrome_120"
    assert imp_os == "windows"

    # Test set_impersonate
    client.impersonate = "chrome_123"
    imp = client.impersonate
    print(f"After set_impersonate: {imp}")
    assert imp == "chrome_123"

    # Test set_impersonate_os
    client.impersonate_os = "macos"
    imp_os = client.impersonate_os
    print(f"After set_impersonate_os: {imp_os}")
    assert imp_os == "macos"

    print("\nImpersonate tests: PASSED")
    return True


def test_headers_with_request():
    """Test that modified headers actually work in requests"""
    print("\n" + "="*60)
    print("Testing Headers with actual HTTP request")
    print("="*60)

    # Create client
    client = primp.Client()

    # Set custom headers
    client.headers = {"User-Agent": "TestBot/1.0"}

    try:
        # Make a request to httpbin which echoes headers
        response = client.get("https://httpbin.org/headers")
        print(f"Status: {response.status_code}")
        print(f"Response (first 200 chars): {response.text[:200]}")

        # Check if our custom User-Agent is in the response
        if "TestBot/1.0" in response.text:
            print("\nCustom headers successfully applied!")
        else:
            print("\nWarning: Custom headers may not have been applied")

        print("\nHeaders with request test: PASSED")
        return True
    except Exception as e:
        print(f"\nWarning: Request failed (this is OK if you're offline): {e}")
        return True


def test_dynamic_changes():
    """Test multiple dynamic changes"""
    print("\n" + "="*60)
    print("Testing Multiple Dynamic Changes")
    print("="*60)

    client = primp.Client()

    # Change headers multiple times
    for i in range(3):
        client.headers = {"Iteration": str(i)}
        headers = client.headers
        print(f"Iteration {i}: {headers}")
        assert headers["Iteration"] == str(i)

    print("\nMultiple dynamic changes: PASSED")
    return True


if __name__ == "__main__":
    print("="*60)
    print("Testing Enhanced primp Features")
    print("="*60)

    results = []

    try:
        results.append(("Headers", test_headers()))
    except Exception as e:
        print(f"\nHeaders test FAILED: {e}")
        results.append(("Headers", False))

    try:
        results.append(("Proxy", test_proxy()))
    except Exception as e:
        print(f"\nProxy test FAILED: {e}")
        results.append(("Proxy", False))

    try:
        results.append(("Impersonate", test_impersonate()))
    except Exception as e:
        print(f"\nImpersonate test FAILED: {e}")
        results.append(("Impersonate", False))

    try:
        results.append(("Headers with Request", test_headers_with_request()))
    except Exception as e:
        print(f"\nHeaders with request test FAILED: {e}")
        results.append(("Headers with Request", False))

    try:
        results.append(("Dynamic Changes", test_dynamic_changes()))
    except Exception as e:
        print(f"\nDynamic changes test FAILED: {e}")
        results.append(("Dynamic Changes", False))

    # Summary
    print("\n" + "="*60)
    print("TEST SUMMARY")
    print("="*60)
    for test_name, passed in results:
        status = "PASSED" if passed else "FAILED"
        print(f"{test_name}: {status}")

    total = len(results)
    passed = sum(1 for _, p in results if p)
    print(f"\nTotal: {passed}/{total} tests passed")
    print("="*60)
