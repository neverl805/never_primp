from __future__ import annotations

import asyncio
import sys
from functools import partial
from typing import TYPE_CHECKING, TypedDict, Iterator
from collections.abc import MutableMapping

if sys.version_info <= (3, 11):
    from typing_extensions import Unpack
else:
    from typing import Unpack


from .never_primp import RClient

if TYPE_CHECKING:
    from .never_primp import IMPERSONATE, IMPERSONATE_OS, ClientRequestParams, HttpMethod, RequestParams, Response
else:

    class _Unpack:
        @staticmethod
        def __getitem__(*args, **kwargs):
            pass

    Unpack = _Unpack()
    RequestParams = ClientRequestParams = TypedDict


class CookieJar(MutableMapping):
    """
    A dict-like container for managing HTTP cookies, compatible with requests.Session.cookies API.

    Examples:
        client = Client()

        # Set cookies
        client.cookies['session_id'] = 'abc123'
        client.cookies['user_token'] = 'xyz789'

        # Get cookies
        value = client.cookies['session_id']
        value = client.cookies.get('user_token', 'default')

        # Update multiple cookies
        client.cookies.update({'key1': 'value1', 'key2': 'value2'})

        # Delete cookies
        del client.cookies['session_id']

        # Clear all cookies
        client.cookies.clear()

        # Check existence
        if 'session_id' in client.cookies:
            print("Session exists")

        # Convert to dict
        all_cookies = dict(client.cookies)
    """

    def __init__(self, client: RClient):
        """Initialize CookieJar with a reference to the client."""
        self._client = client

    def __getitem__(self, name: str) -> str:
        """Get a cookie value by name."""
        value = self._client.get_cookie(name)
        if value is None:
            raise KeyError(name)
        return value

    def __setitem__(self, name: str, value: str) -> None:
        """Set a cookie by name."""
        self._client.set_cookie(name, value)

    def __delitem__(self, name: str) -> None:
        """Delete a cookie by name."""
        # Check if cookie exists first
        if self._client.get_cookie(name) is None:
            raise KeyError(name)
        self._client.delete_cookie(name)

    def __iter__(self) -> Iterator[str]:
        """Iterate over cookie names."""
        return iter(self._client.get_all_cookies().keys())

    def __len__(self) -> int:
        """Return the number of cookies."""
        return len(self._client.get_all_cookies())

    def __contains__(self, name: object) -> bool:
        """Check if a cookie exists."""
        if not isinstance(name, str):
            return False
        return self._client.get_cookie(name) is not None

    def __repr__(self) -> str:
        """Return string representation of cookies."""
        cookies = self._client.get_all_cookies()
        return f"CookieJar({cookies})"

    def get(self, name: str, default: str | None = None) -> str | None:
        """Get a cookie value with a default fallback."""
        value = self._client.get_cookie(name)
        return value if value is not None else default

    def update(self, cookies: dict[str, str], domain: str | None = None, path: str | None = None) -> None:
        """
        Update multiple cookies at once.

        Args:
            cookies: Dictionary of cookie names to values
            domain: Optional domain for the cookies (e.g., ".example.com")
            path: Optional path for the cookies (e.g., "/")
        """
        self._client.update_cookies(cookies, domain=domain, path=path)

    def clear(self) -> None:
        """Remove all cookies."""
        self._client.clear_cookies()

    def set(self, name: str, value: str, domain: str | None = None, path: str | None = None) -> None:
        """
        Set a single cookie with optional domain and path.

        Args:
            name: Cookie name
            value: Cookie value
            domain: Optional domain (e.g., ".example.com")
            path: Optional path (e.g., "/")
        """
        self._client.set_cookie(name, value, domain=domain, path=path)


class Client(RClient):
    """Initializes an HTTP client that can impersonate web browsers."""

    def __init__(
            self,
            auth: tuple[str, str | None] | None = None,
            auth_bearer: str | None = None,
            params: dict[str, str] | None = None,
            headers: dict[str, str] | None = None,
            ordered_headers: dict[str, str] | None = None,
            cookies: dict[str, str] | None = None,
            cookie_store: bool | None = True,
            split_cookies: bool | None = False,
            referer: bool | None = True,
            proxy: str | None = None,
            timeout: float | None = 30,
            impersonate: IMPERSONATE | None = None,
            impersonate_os: IMPERSONATE_OS | None = None,
            follow_redirects: bool | None = True,
            max_redirects: int | None = 20,
            verify: bool | None = True,
            ca_cert_file: str | None = None,
            https_only: bool | None = False,
            http1_only: bool | None = False,
            http2_only: bool | None = False,
            pool_idle_timeout: float | None = None,
            pool_max_idle_per_host: int | None = None,
            tcp_nodelay: bool | None = None,
            tcp_keepalive: float | None = None,
    ):
        """
        Args:
            auth: a tuple containing the username and an optional password for basic authentication. Default is None.
            auth_bearer: a string representing the bearer token for bearer token authentication. Default is None.
            params: a map of query parameters to append to the URL. Default is None.
            headers: an optional map of HTTP headers to send with requests. Ignored if `impersonate` is set.
            ordered_headers: an optional ordered map of HTTP headers with strict order preservation.
                Takes priority over `headers`. Use this for bypassing anti-bot detection that checks header order.
                Example: {"User-Agent": "...", "Accept": "...", "Accept-Language": "..."}
                Note: Python 3.7+ dict maintains insertion order by default.
            cookies: initial cookies to set for the client. These cookies will be included in all requests.
                Can be updated later using client.cookies.update(). Default is None.
            cookie_store: enable a persistent cookie store. Received cookies will be preserved and included
                 in additional requests. Default is True.
            split_cookies: split cookies into multiple `cookie` headers (HTTP/2 style) instead of a single
                `Cookie` header. Useful for mimicking browser behavior in HTTP/2. Default is False.
                When True: cookie: a=1 \n cookie: b=2 \n cookie: c=3
                When False: Cookie: a=1; b=2; c=3
            referer: automatic setting of the `Referer` header. Default is True.
            proxy: proxy URL for HTTP requests, example: "socks5://127.0.0.1:9150". Default is None.
            timeout: timeout for HTTP requests in seconds. Default is 30.
            impersonate: impersonate browser. Supported browsers:
                "chrome_100", "chrome_101", "chrome_104", "chrome_105", "chrome_106",
                "chrome_107", "chrome_108", "chrome_109", "chrome_114", "chrome_116",
                "chrome_117", "chrome_118", "chrome_119", "chrome_120", "chrome_123",
                "chrome_124", "chrome_126", "chrome_127", "chrome_128", "chrome_129",
                "chrome_130", "chrome_131", "chrome_133"
                "safari_15.3", "safari_15.5", "safari_15.6.1", "safari_16",
                "safari_16.5", "safari_17.0", "safari_17.2.1", "safari_17.4.1",
                "safari_17.5", "safari_18",  "safari_18.2",
                "safari_ios_16.5", "safari_ios_17.2", "safari_ios_17.4.1", "safari_ios_18.1.1",
                "safari_ipad_18",
                "okhttp_3.9", "okhttp_3.11", "okhttp_3.13", "okhttp_3.14", "okhttp_4.9",
                "okhttp_4.10", "okhttp_5",
                "edge_101", "edge_122", "edge_127", "edge_131",
                "firefox_109", "firefox_117", "firefox_128", "firefox_133", "firefox_135".
                Default is None.
            impersonate_os: impersonate OS. Supported OS:
                "android", "ios", "linux", "macos", "windows". Default is None.
            follow_redirects: a boolean to enable or disable following redirects. Default is True.
            max_redirects: the maximum number of redirects if `follow_redirects` is True. Default is 20.
            verify: an optional boolean indicating whether to verify SSL certificates. Default is True.
            ca_cert_file: path to CA certificate store. Default is None.
            https_only: restrict the Client to be used with HTTPS only requests. Default is False.
            http1_only: if true - use only HTTP/1.1. Default is False.
            http2_only: if true - use only HTTP/2. Default is False.
                Note: http1_only and http2_only are mutually exclusive. If both are true, http1_only takes precedence.
        """
        super().__init__()
        self._cookies_jar: CookieJar | None = None

        # Set initial cookies if provided
        if cookies:
            self.update_cookies(cookies)

    @property
    def cookies(self) -> CookieJar:
        """
        Access the cookie jar for dict-like cookie management.

        Returns:
            CookieJar: A dict-like container for managing cookies.

        Examples:
            # Set cookies
            client.cookies['session_id'] = 'abc123'

            # Get cookies
            value = client.cookies['session_id']

            # Update cookies
            client.cookies.update({'key': 'value'})

            # Delete cookies
            del client.cookies['session_id']

            # Clear all
            client.cookies.clear()
        """
        if self._cookies_jar is None:
            self._cookies_jar = CookieJar(self)
        return self._cookies_jar

    def __enter__(self) -> Client:
        return self

    def __exit__(self, *args):
        del self

    def request(self, method: HttpMethod, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        """
        Send an HTTP request with support for requests-toolbelt MultipartEncoder.

        Supports both native primp format and requests-toolbelt MultipartEncoder:
        - Native: request(url, data={...}, files={...})
        - Toolbelt: request(url, data=MultipartEncoder(...))
        """
        # Check if data is a MultipartEncoder from requests-toolbelt
        data = kwargs.get('data')
        if data is not None and hasattr(data, 'fields') and hasattr(data, 'content_type'):
            # This looks like a MultipartEncoder
            # Extract fields and convert to primp format
            converted_data = {}
            converted_files = {}

            try:
                # MultipartEncoder.fields is a dict-like object
                for field_name, field_value in data.fields.items():
                    if isinstance(field_value, tuple):
                        # File field: (filename, file_obj, content_type)
                        if len(field_value) >= 2:
                            filename = field_value[0]
                            file_obj = field_value[1]

                            # Read the file content
                            if hasattr(file_obj, 'read'):
                                file_content = file_obj.read()
                                # Reset file pointer if possible
                                if hasattr(file_obj, 'seek'):
                                    try:
                                        file_obj.seek(0)
                                    except:
                                        pass
                            else:
                                file_content = file_obj

                            # Add mime type if provided
                            if len(field_value) >= 3:
                                mime_type = field_value[2]
                                converted_files[field_name] = (filename, file_content, mime_type)
                            else:
                                converted_files[field_name] = (filename, file_content)
                    else:
                        # Regular field
                        if isinstance(field_value, bytes):
                            converted_data[field_name] = field_value.decode('utf-8')
                        else:
                            converted_data[field_name] = str(field_value)

                # Replace data and files in kwargs
                if converted_data:
                    kwargs['data'] = converted_data
                else:
                    kwargs.pop('data', None)

                if converted_files:
                    kwargs['files'] = converted_files

            except Exception as e:
                # If conversion fails, fall back to treating it as raw content
                # Read the encoder as bytes and send as content
                if hasattr(data, 'read'):
                    kwargs['content'] = data.read()
                    kwargs.pop('data', None)

                    # Get content type from encoder
                    if hasattr(data, 'content_type'):
                        headers = kwargs.get('headers', {})
                        if not isinstance(headers, dict):
                            headers = dict(headers)
                        headers['Content-Type'] = data.content_type
                        kwargs['headers'] = headers

        return super().request(method=method, url=url, **kwargs)

    def get(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="GET", url=url, **kwargs)

    def head(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="HEAD", url=url, **kwargs)

    def options(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="OPTIONS", url=url, **kwargs)

    def delete(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="DELETE", url=url, **kwargs)

    def post(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="POST", url=url, **kwargs)

    def put(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="PUT", url=url, **kwargs)

    def patch(self, url: str, **kwargs: Unpack[RequestParams]) -> Response:
        return self.request(method="PATCH", url=url, **kwargs)


class AsyncClient(Client):
    def __init__(self,
                 auth: tuple[str, str | None] | None = None,
                 auth_bearer: str | None = None,
                 params: dict[str, str] | None = None,
                 headers: dict[str, str] | None = None,
                 ordered_headers: dict[str, str] | None = None,
                 cookies: dict[str, str] | None = None,
                 cookie_store: bool | None = True,
                 split_cookies: bool | None = False,
                 referer: bool | None = True,
                 proxy: str | None = None,
                 timeout: float | None = 30,
                 impersonate: IMPERSONATE | None = None,
                 impersonate_os: IMPERSONATE_OS | None = None,
                 follow_redirects: bool | None = True,
                 max_redirects: int | None = 20,
                 verify: bool | None = True,
                 ca_cert_file: str | None = None,
                 https_only: bool | None = False,
                 http1_only: bool | None = False,
                 http2_only: bool | None = False,
                 # Performance optimization parameters
                 pool_idle_timeout: float | None = None,
                 pool_max_idle_per_host: int | None = None,
                 tcp_nodelay: bool | None = None,
                 tcp_keepalive: float | None = None,
                 ):
        super().__init__(
            auth=auth,
            auth_bearer=auth_bearer,
            params=params,
            headers=headers,
            ordered_headers=ordered_headers,
            cookies=cookies,
            cookie_store=cookie_store,
            split_cookies=split_cookies,
            referer=referer,
            proxy=proxy,
            timeout=timeout,
            impersonate=impersonate,
            impersonate_os=impersonate_os,
            follow_redirects=follow_redirects,
            max_redirects=max_redirects,
            verify=verify,
            ca_cert_file=ca_cert_file,
            https_only=https_only,
            http1_only=http1_only,
            http2_only=http2_only,
            pool_idle_timeout=pool_idle_timeout,
            pool_max_idle_per_host=pool_max_idle_per_host,
            tcp_nodelay=tcp_nodelay,
            tcp_keepalive=tcp_keepalive,
        )

    async def __aenter__(self) -> AsyncClient:
        return self

    async def __aexit__(self, *args):
        del self

    async def _run_sync_asyncio(self, fn, *args, **kwargs):
        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, partial(fn, *args, **kwargs))

    async def request(self, method: HttpMethod, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self._run_sync_asyncio(super().request, method=method, url=url, **kwargs)

    async def get(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="GET", url=url, **kwargs)

    async def head(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="HEAD", url=url, **kwargs)

    async def options(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="OPTIONS", url=url, **kwargs)

    async def delete(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="DELETE", url=url, **kwargs)

    async def post(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="POST", url=url, **kwargs)

    async def put(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="PUT", url=url, **kwargs)

    async def patch(self, url: str, **kwargs: Unpack[RequestParams]):  # type: ignore
        return await self.request(method="PATCH", url=url, **kwargs)


def request(
        method: HttpMethod,
        url: str,
        impersonate: IMPERSONATE | None = None,
        impersonate_os: IMPERSONATE_OS | None = None,
        verify: bool | None = True,
        ca_cert_file: str | None = None,
        **kwargs: Unpack[RequestParams],
):
    """
    Args:
        method: the HTTP method to use (e.g., "GET", "POST").
        url: the URL to which the request will be made.
        impersonate: impersonate browser. Supported browsers:
            "chrome_100", "chrome_101", "chrome_104", "chrome_105", "chrome_106",
            "chrome_107", "chrome_108", "chrome_109", "chrome_114", "chrome_116",
            "chrome_117", "chrome_118", "chrome_119", "chrome_120", "chrome_123",
            "chrome_124", "chrome_126", "chrome_127", "chrome_128", "chrome_129",
            "chrome_130", "chrome_131", "chrome_133",
            "safari_15.3", "safari_15.5", "safari_15.6.1", "safari_16",
            "safari_16.5", "safari_17.0", "safari_17.2.1", "safari_17.4.1",
            "safari_17.5", "safari_18",  "safari_18.2",
            "safari_ios_16.5", "safari_ios_17.2", "safari_ios_17.4.1", "safari_ios_18.1.1",
            "safari_ipad_18",
            "okhttp_3.9", "okhttp_3.11", "okhttp_3.13", "okhttp_3.14", "okhttp_4.9",
            "okhttp_4.10", "okhttp_5",
            "edge_101", "edge_122", "edge_127", "edge_131",
            "firefox_109", "firefox_117", "firefox_128", "firefox_133", "firefox_135".
            Default is None.
        impersonate_os: impersonate OS. Supported OS:
            "android", "ios", "linux", "macos", "windows". Default is None.
        verify: an optional boolean indicating whether to verify SSL certificates. Default is True.
        ca_cert_file: path to CA certificate store. Default is None.
        auth: a tuple containing the username and an optional password for basic authentication. Default is None.
        auth_bearer: a string representing the bearer token for bearer token authentication. Default is None.
        params: a map of query parameters to append to the URL. Default is None.
        headers: an optional map of HTTP headers to send with requests. If `impersonate` is set, this will be ignored.
        cookies: an optional map of cookies to send with requests as the `Cookie` header.
        timeout: the timeout for the request in seconds. Default is 30.
        content: the content to send in the request body as bytes. Default is None.
        data: the form data to send in the request body. Default is None.
        json: a JSON serializable object to send in the request body. Default is None.
        files: a map of file fields to file paths to be sent as multipart/form-data. Default is None.
    """
    with Client(
            impersonate=impersonate,
            impersonate_os=impersonate_os,
            verify=verify,
            ca_cert_file=ca_cert_file,
    ) as client:
        return client.request(method, url, **kwargs)


def get(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="GET", url=url, **kwargs)


def head(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="HEAD", url=url, **kwargs)


def options(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="OPTIONS", url=url, **kwargs)


def delete(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="DELETE", url=url, **kwargs)


def post(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="POST", url=url, **kwargs)


def put(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="PUT", url=url, **kwargs)


def patch(url: str, **kwargs: Unpack[ClientRequestParams]):
    return request(method="PATCH", url=url, **kwargs)
