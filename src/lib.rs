#![allow(clippy::too_many_arguments)]
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use anyhow::Result;
use foldhash::fast::RandomState;
use indexmap::IndexMap;
use pyo3::prelude::*;
use pythonize::depythonize;
use wreq::{
    multipart,
    redirect::Policy,
    Body, Method,
};
use wreq_util::{Emulation, EmulationOS, EmulationOption};
use serde_json::Value;
use tokio::{
    fs::File,
    runtime::{self, Runtime},
};
use tokio_util::codec::{BytesCodec, FramedRead};
use tracing;

mod impersonate;
use impersonate::{ImpersonateFromStr, ImpersonateOSFromStr};
mod response;
use response::Response;

mod traits;
use traits::HeadersTraits;

mod utils;
use utils::load_ca_certs;

type IndexMapSSR = IndexMap<String, String, RandomState>;

// Tokio global one-thread runtime
static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
});

#[pyclass(subclass)]
/// HTTP client that can impersonate web browsers.
pub struct RClient {
    client: Arc<Mutex<wreq::Client>>,
    // Cookie jar for manual cookie management
    cookie_jar: Arc<wreq::cookie::Jar>,
    #[pyo3(get, set)]
    auth: Option<(String, Option<String>)>,
    #[pyo3(get, set)]
    auth_bearer: Option<String>,
    #[pyo3(get, set)]
    params: Option<IndexMapSSR>,
    #[pyo3(get, set)]
    proxy: Option<String>,
    #[pyo3(get, set)]
    timeout: Option<f64>,
    #[pyo3(get)]
    impersonate: Option<String>,
    #[pyo3(get)]
    impersonate_os: Option<String>,
    // Configuration fields for client rebuild
    headers: Arc<Mutex<Option<IndexMapSSR>>>,
    ordered_headers: Arc<Mutex<Option<IndexMapSSR>>>,
    cookie_store: Option<bool>,
    #[pyo3(get, set)]
    split_cookies: Option<bool>,
    referer: Option<bool>,
    follow_redirects: Option<bool>,
    max_redirects: Option<usize>,
    verify: Option<bool>,
    ca_cert_file: Option<String>,
    https_only: Option<bool>,
    http2_only: Option<bool>,
    // Performance optimization fields
    pool_idle_timeout: Option<f64>,
    pool_max_idle_per_host: Option<usize>,
    tcp_nodelay: Option<bool>,
    tcp_keepalive: Option<f64>,
    // Retry mechanism
    #[pyo3(get, set)]
    retry_count: Option<usize>,
    #[pyo3(get, set)]
    retry_backoff: Option<f64>,
}

#[pymethods]
impl RClient {
    /// Initializes an HTTP client that can impersonate web browsers.
    ///
    /// This function creates a new HTTP client instance that can impersonate various web browsers.
    /// It allows for customization of headers, proxy settings, timeout, impersonation type, SSL certificate verification,
    /// and HTTP version preferences.
    ///
    /// # Arguments
    ///
    /// * `auth` - A tuple containing the username and an optional password for basic authentication. Default is None.
    /// * `auth_bearer` - A string representing the bearer token for bearer token authentication. Default is None.
    /// * `params` - A map of query parameters to append to the URL. Default is None.
    /// * `headers` - An optional map of HTTP headers to send with requests. If `impersonate` is set, this will be ignored.
    /// * `ordered_headers` - An optional ordered map of HTTP headers with strict order preservation. Takes priority over `headers`.
    /// * `cookie_store` - Enable a persistent cookie store. Received cookies will be preserved and included
    ///         in additional requests. Default is `true`.
    /// * `split_cookies` - Split cookies into multiple `cookie` headers (HTTP/2 style) instead of a single `Cookie` header.
    ///         Useful for mimicking browser behavior in HTTP/2. Default is `false`.
    /// * `referer` - Enable or disable automatic setting of the `Referer` header. Default is `true`.
    /// * `proxy` - An optional proxy URL for HTTP requests.
    /// * `timeout` - An optional timeout for HTTP requests in seconds.
    /// * `impersonate` - An optional entity to impersonate. Supported browsers and versions include Chrome, Safari, OkHttp, and Edge.
    /// * `impersonate_os` - An optional entity to impersonate OS. Supported OS: android, ios, linux, macos, windows.
    /// * `follow_redirects` - A boolean to enable or disable following redirects. Default is `true`.
    /// * `max_redirects` - The maximum number of redirects to follow. Default is 20. Applies if `follow_redirects` is `true`.
    /// * `verify` - An optional boolean indicating whether to verify SSL certificates. Default is `true`.
    /// * `ca_cert_file` - Path to CA certificate store. Default is None.
    /// * `https_only` - Restrict the Client to be used with HTTPS only requests. Default is `false`.
    /// * `http2_only` - If true - use only HTTP/2, if false - use only HTTP/1. Default is `false`.
    ///
    /// # Example
    ///
    /// ```
    /// from primp import Client
    ///
    /// client = Client(
    ///     auth=("name", "password"),
    ///     params={"p1k": "p1v", "p2k": "p2v"},
    ///     headers={"User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/88.0.4324.150 Safari/537.36"},
    ///     cookie_store=False,
    ///     referer=False,
    ///     proxy="http://127.0.0.1:8080",
    ///     timeout=10,
    ///     impersonate="chrome_123",
    ///     impersonate_os="windows",
    ///     follow_redirects=True,
    ///     max_redirects=1,
    ///     verify=True,
    ///     ca_cert_file="/cert/cacert.pem",
    ///     https_only=True,
    ///     http2_only=True,
    /// )
    /// ```
    #[new]
    #[pyo3(signature = (auth=None, auth_bearer=None, params=None, headers=None, ordered_headers=None, cookie_store=true,
        split_cookies=false, referer=true, proxy=None, timeout=None, impersonate=None, impersonate_os=None, follow_redirects=true,
        max_redirects=20, verify=true, ca_cert_file=None, https_only=false, http2_only=false,
        pool_idle_timeout=None, pool_max_idle_per_host=None, tcp_nodelay=None, tcp_keepalive=None,
        retry_count=None, retry_backoff=None))]
    fn new(
        auth: Option<(String, Option<String>)>,
        auth_bearer: Option<String>,
        params: Option<IndexMapSSR>,
        headers: Option<IndexMapSSR>,
        ordered_headers: Option<IndexMapSSR>,
        cookie_store: Option<bool>,
        split_cookies: Option<bool>,
        referer: Option<bool>,
        proxy: Option<String>,
        timeout: Option<f64>,
        impersonate: Option<String>,
        impersonate_os: Option<String>,
        follow_redirects: Option<bool>,
        max_redirects: Option<usize>,
        verify: Option<bool>,
        ca_cert_file: Option<String>,
        https_only: Option<bool>,
        http2_only: Option<bool>,
        // Performance optimization parameters
        pool_idle_timeout: Option<f64>,
        pool_max_idle_per_host: Option<usize>,
        tcp_nodelay: Option<bool>,
        tcp_keepalive: Option<f64>,
        // Retry mechanism
        retry_count: Option<usize>,
        retry_backoff: Option<f64>,
    ) -> Result<Self> {
        // Client builder
        let mut client_builder = wreq::Client::builder();

        // Impersonate
        if let Some(impersonate) = &impersonate {
            let imp = Emulation::from_str(&impersonate.as_str())?;
            let imp_os = if let Some(impersonate_os) = &impersonate_os {
                EmulationOS::from_str(&impersonate_os.as_str())?
            } else {
                EmulationOS::default()
            };
            let emulation_option = EmulationOption::builder()
                .emulation(imp)
                .emulation_os(imp_os)
                .build();
            client_builder = client_builder.emulation(emulation_option);
        }

        // Headers - prioritize ordered_headers over regular headers
        if let Some(ref ordered_hdrs) = ordered_headers {
            // Use ordered headers with OrigHeaderMap for strict order preservation
            let headers_headermap = ordered_hdrs.to_headermap();
            let orig_headermap = ordered_hdrs.to_orig_headermap();
            client_builder = client_builder
                .default_headers(headers_headermap)
                .orig_headers(orig_headermap);
        } else if let Some(ref hdrs) = headers {
            // Fallback to regular headers
            let headers_headermap = hdrs.to_headermap();
            client_builder = client_builder.default_headers(headers_headermap);
        };

        // Cookie jar - create and configure
        let cookie_jar = Arc::new(wreq::cookie::Jar::default());

        // Cookie_store
        if cookie_store.unwrap_or(true) {
            client_builder = client_builder.cookie_provider(cookie_jar.clone());
        }

        // Referer
        if referer.unwrap_or(true) {
            client_builder = client_builder.referer(true);
        }

        // Proxy
        let proxy = proxy.or_else(|| std::env::var("PRIMP_PROXY").ok());
        if let Some(proxy) = &proxy {
            client_builder = client_builder.proxy(wreq::Proxy::all(proxy)?);
        }

        // Timeout
        if let Some(seconds) = timeout {
            client_builder = client_builder.timeout(Duration::from_secs_f64(seconds));
        }

        // Redirects
        if follow_redirects.unwrap_or(true) {
            client_builder = client_builder.redirect(Policy::limited(max_redirects.unwrap_or(20)));
        } else {
            client_builder = client_builder.redirect(Policy::none());
        }

        // Ca_cert_file. BEFORE!!! verify (fn load_ca_certs() reads env var PRIMP_CA_BUNDLE)
        if let Some(ca_bundle_path) = &ca_cert_file {
            unsafe {
                std::env::set_var("PRIMP_CA_BUNDLE", ca_bundle_path);
            }
        }

        // Verify
        if verify.unwrap_or(true) {
            if let Some(cert_store) = load_ca_certs() {
                client_builder = client_builder.cert_store(cert_store.clone());
            }
        } else {
            client_builder = client_builder.cert_verification(false);
        }

        // Https_only
        if let Some(true) = https_only {
            client_builder = client_builder.https_only(true);
        }

        // Http2_only
        if let Some(true) = http2_only {
            client_builder = client_builder.http2_only();
        }

        // Performance optimization: Connection pool settings
        if let Some(timeout) = pool_idle_timeout {
            client_builder = client_builder.pool_idle_timeout(Duration::from_secs_f64(timeout));
        }

        if let Some(max_idle) = pool_max_idle_per_host {
            client_builder = client_builder.pool_max_idle_per_host(max_idle);
        }

        // TCP optimization
        if let Some(true) = tcp_nodelay {
            client_builder = client_builder.tcp_nodelay(true);
        }

        if let Some(interval) = tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(Some(Duration::from_secs_f64(interval)));
        }

        let client = Arc::new(Mutex::new(client_builder.build()?));

        Ok(RClient {
            client,
            cookie_jar,
            auth,
            auth_bearer,
            params,
            proxy,
            timeout,
            impersonate,
            impersonate_os,
            // Store configuration for potential client rebuild
            headers: Arc::new(Mutex::new(headers)),
            ordered_headers: Arc::new(Mutex::new(ordered_headers)),
            cookie_store,
            split_cookies,
            referer,
            follow_redirects,
            max_redirects,
            verify,
            ca_cert_file,
            https_only,
            http2_only,
            // Performance optimization fields
            pool_idle_timeout,
            pool_max_idle_per_host,
            tcp_nodelay,
            tcp_keepalive,
            // Retry mechanism
            retry_count,
            retry_backoff,
        })
    }

    #[getter]
    pub fn get_headers(&self) -> Result<IndexMapSSR> {
        if let Ok(headers_guard) = self.headers.lock() {
            Ok(headers_guard.clone().unwrap_or_else(|| IndexMap::with_capacity_and_hasher(10, RandomState::default())))
        } else {
            Ok(IndexMap::with_capacity_and_hasher(10, RandomState::default()))
        }
    }

    #[setter]
    pub fn set_headers(&mut self, new_headers: Option<IndexMapSSR>) -> Result<()> {
        if let Ok(mut headers_guard) = self.headers.lock() {
            *headers_guard = new_headers;
        }
        self.rebuild_client()?;
        Ok(())
    }

    pub fn headers_update(&mut self, new_headers: Option<IndexMapSSR>) -> Result<()> {
        if let Some(new_headers) = new_headers {
            if let Ok(mut headers_guard) = self.headers.lock() {
                if let Some(existing_headers) = headers_guard.as_mut() {
                    // Update existing headers
                    for (key, value) in new_headers {
                        existing_headers.insert(key, value);
                    }
                } else {
                    // No existing headers, set new ones
                    *headers_guard = Some(new_headers);
                }
            }
            self.rebuild_client()?;
        }
        Ok(())
    }

    #[getter]
    pub fn get_ordered_headers(&self) -> Result<IndexMapSSR> {
        if let Ok(headers_guard) = self.ordered_headers.lock() {
            Ok(headers_guard.clone().unwrap_or_else(|| IndexMap::with_capacity_and_hasher(10, RandomState::default())))
        } else {
            Ok(IndexMap::with_capacity_and_hasher(10, RandomState::default()))
        }
    }

    #[setter]
    pub fn set_ordered_headers(&mut self, new_headers: Option<IndexMapSSR>) -> Result<()> {
        if let Ok(mut headers_guard) = self.ordered_headers.lock() {
            *headers_guard = new_headers;
        }
        self.rebuild_client()?;
        Ok(())
    }

    pub fn ordered_headers_update(&mut self, new_headers: Option<IndexMapSSR>) -> Result<()> {
        if let Some(new_headers) = new_headers {
            if let Ok(mut headers_guard) = self.ordered_headers.lock() {
                if let Some(existing_headers) = headers_guard.as_mut() {
                    // Update existing ordered headers (preserves insertion order)
                    for (key, value) in new_headers {
                        existing_headers.insert(key, value);
                    }
                } else {
                    // No existing ordered headers, set new ones
                    *headers_guard = Some(new_headers);
                }
            }
            self.rebuild_client()?;
        }
        Ok(())
    }

    #[getter]
    pub fn get_proxy(&self) -> Result<Option<String>> {
        Ok(self.proxy.to_owned())
    }

    #[setter]
    pub fn set_proxy(&mut self, proxy: String) -> Result<()> {
        self.proxy = Some(proxy);
        self.rebuild_client()?;
        Ok(())
    }

    #[setter]
    pub fn set_impersonate(&mut self, impersonate: String) -> Result<()> {
        self.impersonate = Some(impersonate);
        self.rebuild_client()?;
        Ok(())
    }

    #[setter]
    pub fn set_impersonate_os(&mut self, impersonate_os: String) -> Result<()> {
        self.impersonate_os = Some(impersonate_os);
        self.rebuild_client()?;
        Ok(())
    }

    #[pyo3(signature = (url))]
    #[allow(unused_variables)]
    fn get_cookies(&self, url: &str) -> Result<IndexMapSSR> {
        let mut cookies = IndexMap::with_capacity_and_hasher(10, RandomState::default());

        // Get all cookies from the jar
        // Note: wreq's get_all() returns all cookies regardless of URL
        // To get URL-specific cookies, you would need to filter manually or use get(name, uri) per cookie
        for cookie in self.cookie_jar.get_all() {
            cookies.insert(cookie.name().to_string(), cookie.value().to_string());
        }

        Ok(cookies)
    }

    #[pyo3(signature = (url, cookies))]
    fn set_cookies(&self, url: &str, cookies: Option<IndexMapSSR>) -> Result<()> {
        if let Some(cookies) = cookies {
            let uri: wreq::Uri = url.parse()?;

            for (name, value) in cookies {
                // Format as "name=value" cookie string
                let cookie_str = format!("{}={}", name, value);
                self.cookie_jar.add_cookie_str(&cookie_str, &uri);
            }
        }
        Ok(())
    }

    /// Constructs an HTTP request with the given method, URL, and optionally sets a timeout, headers, and query parameters.
    /// Sends the request and returns a `Response` object containing the server's response.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method to use (e.g., "GET", "POST").
    /// * `url` - The URL to which the request will be made.
    /// * `params` - A map of query parameters to append to the URL. Default is None.
    /// * `headers` - A map of HTTP headers to send with the request. Default is None.
    /// * `cookies` - An optional map of cookies to send with requests as the `Cookie` header.
    /// * `content` - The content to send in the request body as bytes. Default is None.
    /// * `data` - The form data to send in the request body. Default is None.
    /// * `json` -  A JSON serializable object to send in the request body. Default is None.
    /// * `files` - A map of file fields to file paths to be sent as multipart/form-data. Default is None.
    /// * `auth` - A tuple containing the username and an optional password for basic authentication. Default is None.
    /// * `auth_bearer` - A string representing the bearer token for bearer token authentication. Default is None.
    /// * `timeout` - The timeout for the request in seconds. Default is 30.
    ///
    /// # Returns
    ///
    /// * `Response` - A response object containing the server's response to the request.
    ///
    /// # Errors
    ///
    /// * `PyException` - If there is an error making the request.
    #[pyo3(signature = (method, url, params=None, headers=None, ordered_headers=None, cookies=None, content=None,
        data=None, json=None, files=None, auth=None, auth_bearer=None, timeout=None))]
    fn request(
        &self,
        py: Python,
        method: &str,
        url: &str,
        params: Option<IndexMapSSR>,
        headers: Option<IndexMapSSR>,
        ordered_headers: Option<IndexMapSSR>,
        cookies: Option<IndexMapSSR>,
        content: Option<Vec<u8>>,
        data: Option<&Bound<'_, PyAny>>,
        json: Option<&Bound<'_, PyAny>>,
        files: Option<IndexMap<String, String>>,
        auth: Option<(String, Option<String>)>,
        auth_bearer: Option<String>,
        timeout: Option<f64>,
    ) -> Result<Response> {
        let client = Arc::clone(&self.client);
        let method = Method::from_bytes(method.as_bytes())?;
        let is_post_put_patch = matches!(method, Method::POST | Method::PUT | Method::PATCH);
        let params = params.or_else(|| self.params.clone());
        let data_value: Option<Value> = data.map(depythonize).transpose()?;
        let json_value: Option<Value> = json.map(depythonize).transpose()?;
        let auth = auth.or(self.auth.clone());
        let auth_bearer = auth_bearer.or(self.auth_bearer.clone());
        let timeout: Option<f64> = timeout.or(self.timeout);

        let future = async {
            // Create request builder
            let mut request_builder = client.lock().unwrap().request(method, url);

            // Params
            if let Some(params) = params {
                request_builder = request_builder.query(&params);
            }

            // Headers - prioritize ordered_headers over regular headers
            if let Some(ordered_hdrs) = ordered_headers {
                // Use ordered headers with OrigHeaderMap for strict order preservation
                let headers_headermap = ordered_hdrs.to_headermap();
                let orig_headermap = ordered_hdrs.to_orig_headermap();
                request_builder = request_builder
                    .headers(headers_headermap)
                    .orig_headers(orig_headermap);
            } else if let Some(headers) = headers {
                // Fallback to regular headers
                request_builder = request_builder.headers(headers.to_headermap());
            }

            // Cookies - handle based on split_cookies option
            if let Some(cookies) = cookies {
                if !cookies.is_empty() {
                    if self.split_cookies.unwrap_or(false) {
                        // Split: multiple cookie headers (HTTP/2 style)
                        // Use lowercase "cookie" and header_append for multiple headers
                        for (k, v) in cookies.iter() {
                            let cookie_value = format!("{}={}", k, v);
                            request_builder = request_builder.header_append("cookie", cookie_value);
                        }
                    } else {
                        // Merge: single Cookie header (HTTP/1.1 style, default)
                        let cookie_value = cookies
                            .iter()
                            .map(|(k, v)| format!("{}={}", k, v))
                            .collect::<Vec<_>>()
                            .join("; ");
                        request_builder = request_builder.header("Cookie", cookie_value);
                    }
                }
            }

            // Only if method POST || PUT || PATCH
            if is_post_put_patch {
                // Content
                if let Some(content) = content {
                    request_builder = request_builder.body(content);
                }
                // Data
                if let Some(form_data) = data_value {
                    request_builder = request_builder.form(&form_data);
                }
                // Json
                if let Some(json_data) = json_value {
                    request_builder = request_builder.json(&json_data);
                }
                // Files
                if let Some(files) = files {
                    let mut form = multipart::Form::new();
                    for (file_name, file_path) in files {
                        let file = File::open(file_path).await?;
                        let stream = FramedRead::new(file, BytesCodec::new());
                        let file_body = Body::wrap_stream(stream);
                        let part = multipart::Part::stream(file_body).file_name(file_name.clone());
                        form = form.part(file_name, part);
                    }
                    request_builder = request_builder.multipart(form);
                }
            }

            // Auth
            if let Some((username, password)) = auth {
                request_builder = request_builder.basic_auth(username, password);
            } else if let Some(token) = auth_bearer {
                request_builder = request_builder.bearer_auth(token);
            }

            // Timeout
            if let Some(seconds) = timeout {
                request_builder = request_builder.timeout(Duration::from_secs_f64(seconds));
            }

            // Send the request and await the response
            let resp: wreq::Response = request_builder.send().await?;
            let url: String = resp.uri().to_string();
            let status_code = resp.status().as_u16();

            tracing::info!("response: {} {}", url, status_code);
            Ok((resp, url, status_code))
        };

        // Execute an async future, releasing the Python GIL for concurrency.
        // Use Tokio global runtime to block on the future.
        let response: Result<(wreq::Response, String, u16)> =
            py.allow_threads(|| RUNTIME.block_on(future));
        let result = response?;
        let resp = http::Response::from(result.0);
        let url = result.1;
        let status_code = result.2;
        Ok(Response {
            resp,
            _content: None,
            _encoding: None,
            _headers: None,
            _cookies: None,
            url,
            status_code,
        })
    }
}

// Internal implementation (not exposed to Python)
impl RClient {
    /// Rebuilds the wreq client with current configuration
    fn rebuild_client(&mut self) -> Result<()> {
        let mut client_builder = wreq::Client::builder();

        // Impersonate
        if let Some(impersonate) = &self.impersonate {
            let imp = Emulation::from_str(&impersonate.as_str())?;
            let imp_os = if let Some(impersonate_os) = &self.impersonate_os {
                EmulationOS::from_str(&impersonate_os.as_str())?
            } else {
                EmulationOS::default()
            };
            let emulation_option = EmulationOption::builder()
                .emulation(imp)
                .emulation_os(imp_os)
                .build();
            client_builder = client_builder.emulation(emulation_option);
        }

        // Headers - prioritize ordered_headers over regular headers
        if let Ok(ordered_guard) = self.ordered_headers.lock() {
            if let Some(ordered_hdrs) = ordered_guard.as_ref() {
                // Use ordered headers with OrigHeaderMap for strict order preservation
                let headers_headermap = ordered_hdrs.to_headermap();
                let orig_headermap = ordered_hdrs.to_orig_headermap();
                client_builder = client_builder
                    .default_headers(headers_headermap)
                    .orig_headers(orig_headermap);
            } else if let Ok(headers_guard) = self.headers.lock() {
                if let Some(headers) = headers_guard.as_ref() {
                    // Fallback to regular headers
                    let headers_headermap = headers.to_headermap();
                    client_builder = client_builder.default_headers(headers_headermap);
                }
            }
        } else if let Ok(headers_guard) = self.headers.lock() {
            if let Some(headers) = headers_guard.as_ref() {
                // Fallback to regular headers
                let headers_headermap = headers.to_headermap();
                client_builder = client_builder.default_headers(headers_headermap);
            }
        }

        // Cookie_store
        if self.cookie_store.unwrap_or(true) {
            client_builder = client_builder.cookie_provider(self.cookie_jar.clone());
        }

        // Referer
        if self.referer.unwrap_or(true) {
            client_builder = client_builder.referer(true);
        }

        // Proxy
        let proxy = self.proxy.clone().or_else(|| std::env::var("PRIMP_PROXY").ok());
        if let Some(proxy) = &proxy {
            client_builder = client_builder.proxy(wreq::Proxy::all(proxy)?);
        }

        // Timeout
        if let Some(seconds) = self.timeout {
            client_builder = client_builder.timeout(Duration::from_secs_f64(seconds));
        }

        // Redirects
        if self.follow_redirects.unwrap_or(true) {
            client_builder = client_builder.redirect(Policy::limited(self.max_redirects.unwrap_or(20)));
        } else {
            client_builder = client_builder.redirect(Policy::none());
        }

        // Ca_cert_file
        if let Some(ca_bundle_path) = &self.ca_cert_file {
            unsafe {
                std::env::set_var("PRIMP_CA_BUNDLE", ca_bundle_path);
            }
        }

        // Verify
        if self.verify.unwrap_or(true) {
            if let Some(cert_store) = load_ca_certs() {
                client_builder = client_builder.cert_store(cert_store.clone());
            }
        } else {
            client_builder = client_builder.cert_verification(false);
        }

        // Https_only
        if let Some(true) = self.https_only {
            client_builder = client_builder.https_only(true);
        }

        // Http2_only
        if let Some(true) = self.http2_only {
            client_builder = client_builder.http2_only();
        }

        // Performance optimization: Connection pool settings
        if let Some(timeout) = self.pool_idle_timeout {
            client_builder = client_builder.pool_idle_timeout(Duration::from_secs_f64(timeout));
        }

        if let Some(max_idle) = self.pool_max_idle_per_host {
            client_builder = client_builder.pool_max_idle_per_host(max_idle);
        }

        // TCP optimization
        if let Some(true) = self.tcp_nodelay {
            client_builder = client_builder.tcp_nodelay(true);
        }

        if let Some(interval) = self.tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(Some(Duration::from_secs_f64(interval)));
        }

        // Build and replace client
        let new_client = client_builder.build()?;
        if let Ok(mut client_guard) = self.client.lock() {
            *client_guard = new_client;
        }

        Ok(())
    }
}

#[pymodule]
fn never_primp(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    pyo3_log::init();

    m.add_class::<RClient>()?;
    Ok(())
}
