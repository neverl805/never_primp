#![allow(clippy::too_many_arguments)]
use std::sync::{Arc, LazyLock, Mutex};
use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use foldhash::fast::RandomState;
use indexmap::IndexMap;
use pyo3::prelude::*;
use pythonize::depythonize;
use wreq::{
    header::OrigHeaderMap,
    multipart,
    redirect::Policy,
    Body, Method,
};
use wreq_util::{Emulation, EmulationOS, EmulationOption};
use serde_json::Value;
use serde_urlencoded;
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
    // Track deleted cookies (workaround for wreq not filtering expired cookies in get_all())
    deleted_cookies: Arc<Mutex<std::collections::HashSet<String>>>,
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
    http1_only: Option<bool>,
    http2_only: Option<bool>,
    // Performance optimization fields
    pool_idle_timeout: Option<f64>,
    pool_max_idle_per_host: Option<usize>,
    tcp_nodelay: Option<bool>,
    tcp_keepalive: Option<f64>,
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
    /// * `http1_only` - If true - use only HTTP/1.1. Default is `false`.
    /// * `http2_only` - If true - use only HTTP/2. Default is `false`.
    ///   Note: `http1_only` and `http2_only` are mutually exclusive. If both are true, `http1_only` takes precedence.
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
    #[pyo3(signature = (auth=None, auth_bearer=None, params=None, headers=None, ordered_headers=None, cookies=None, cookie_store=true,
        split_cookies=false, referer=true, proxy=None, timeout=None, impersonate=None, impersonate_os=None, follow_redirects=true,
        max_redirects=20, verify=true, ca_cert_file=None, https_only=false, http1_only=false, http2_only=false,
        pool_idle_timeout=None, pool_max_idle_per_host=None, tcp_nodelay=None, tcp_keepalive=None))]
    fn new(
        auth: Option<(String, Option<String>)>,
        auth_bearer: Option<String>,
        params: Option<IndexMapSSR>,
        headers: Option<IndexMapSSR>,
        ordered_headers: Option<IndexMapSSR>,
        cookies: Option<IndexMapSSR>,
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
        http1_only: Option<bool>,
        http2_only: Option<bool>,
        // Performance optimization parameters
        pool_idle_timeout: Option<f64>,
        pool_max_idle_per_host: Option<usize>,
        tcp_nodelay: Option<bool>,
        tcp_keepalive: Option<f64>,
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
        if let Some(ref _ordered_hdrs) = ordered_headers {
            // Don't set ordered_headers as default_headers to avoid duplication
            // They will be applied dynamically at request time with proper ordering
            // (including Host, Content-Length repositioning)
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

        // Http1_only and Http2_only (mutually exclusive, http1_only takes precedence)
        if let Some(true) = http1_only {
            client_builder = client_builder.http1_only();
        } else if let Some(true) = http2_only {
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

        let rclient = RClient {
            client,
            cookie_jar,
            deleted_cookies: Arc::new(Mutex::new(HashSet::new())),
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
            http1_only,
            http2_only,
            // Performance optimization fields
            pool_idle_timeout,
            pool_max_idle_per_host,
            tcp_nodelay,
            tcp_keepalive,
        };

        // Set initial cookies if provided
        if let Some(init_cookies) = cookies {
            rclient.update_cookies(init_cookies, None, None)?;
        }

        Ok(rclient)
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

    /// Get all cookies from the jar without requiring a URL.
    /// Returns a dictionary of cookie names to values.
    fn get_all_cookies(&self) -> Result<IndexMapSSR> {
        let mut cookies = IndexMap::with_capacity_and_hasher(10, RandomState::default());
        let deleted = self.deleted_cookies.lock().unwrap();

        for cookie in self.cookie_jar.get_all() {
            let name = cookie.name();
            // Filter out deleted cookies
            if !deleted.contains(name) {
                cookies.insert(name.to_string(), cookie.value().to_string());
            }
        }
        Ok(cookies)
    }

    /// Set a single cookie without requiring a URL.
    ///
    /// # Arguments
    /// * `name` - Cookie name
    /// * `value` - Cookie value
    /// * `domain` - Optional domain (e.g., ".example.com"). If None, uses a wildcard domain.
    /// * `path` - Optional path (e.g., "/"). If None, uses "/".
    #[pyo3(signature = (name, value, domain=None, path=None))]
    fn set_cookie(
        &self,
        name: String,
        value: String,
        domain: Option<String>,
        path: Option<String>,
    ) -> Result<()> {
        let domain = domain.unwrap_or_else(|| "0.0.0.0".to_string());
        let path = path.unwrap_or_else(|| "/".to_string());

        // Construct a URL from domain and path
        let url = format!("http://{}{}", domain, path);
        let uri: wreq::Uri = url.parse()?;

        let cookie_str = format!("{}={}", name, value);
        self.cookie_jar.add_cookie_str(&cookie_str, &uri);

        // Remove from deleted list
        self.deleted_cookies.lock().unwrap().remove(&name);
        Ok(())
    }

    /// Get a single cookie value by name.
    /// Returns None if the cookie doesn't exist.
    #[pyo3(signature = (name))]
    fn get_cookie(&self, name: String) -> Result<Option<String>> {
        // Check if deleted
        if self.deleted_cookies.lock().unwrap().contains(&name) {
            return Ok(None);
        }

        for cookie in self.cookie_jar.get_all() {
            if cookie.name() == name {
                return Ok(Some(cookie.value().to_string()));
            }
        }
        Ok(None)
    }

    /// Update multiple cookies at once without requiring a URL.
    ///
    /// # Arguments
    /// * `cookies` - Dictionary of cookie names to values
    /// * `domain` - Optional domain. If None, uses a wildcard domain.
    /// * `path` - Optional path. If None, uses "/".
    #[pyo3(signature = (cookies, domain=None, path=None))]
    fn update_cookies(
        &self,
        cookies: IndexMapSSR,
        domain: Option<String>,
        path: Option<String>,
    ) -> Result<()> {
        let domain = domain.unwrap_or_else(|| "0.0.0.0".to_string());
        let path = path.unwrap_or_else(|| "/".to_string());

        let url = format!("http://{}{}", domain, path);
        let uri: wreq::Uri = url.parse()?;

        let mut deleted = self.deleted_cookies.lock().unwrap();
        for (name, value) in cookies {
            let cookie_str = format!("{}={}", name, value);
            self.cookie_jar.add_cookie_str(&cookie_str, &uri);
            // Remove from deleted list
            deleted.remove(&name);
        }
        Ok(())
    }

    /// Delete a single cookie by name.
    /// Sets the cookie to an empty value with Max-Age=0 to delete it.
    #[pyo3(signature = (name))]
    fn delete_cookie(&self, name: String) -> Result<()> {
        // To delete a cookie, set it with an expiration in the past
        let url = "http://0.0.0.0/";
        let uri: wreq::Uri = url.parse()?;

        // Set cookie with Max-Age=0 to delete it
        let cookie_str = format!("{}=; Max-Age=0", name);
        self.cookie_jar.add_cookie_str(&cookie_str, &uri);

        // Add to deleted list
        self.deleted_cookies.lock().unwrap().insert(name);
        Ok(())
    }

    /// Clear all cookies from the jar.
    /// Sets all cookies with Max-Age=0 to mark them as expired.
    fn clear_cookies(&self) -> Result<()> {
        // Get all cookie names first to avoid borrow issues
        let cookie_names: Vec<String> = self.cookie_jar
            .get_all()
            .map(|c| c.name().to_string())
            .collect();

        // Set each cookie with Expires in the past to mark as deleted
        let url = "http://0.0.0.0/";
        let uri: wreq::Uri = url.parse()?;

        let mut deleted = self.deleted_cookies.lock().unwrap();
        for name in cookie_names {
            // Use Expires with a date in the past (Unix epoch)
            let cookie_str = format!("{}=; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Max-Age=0", name);
            self.cookie_jar.add_cookie_str(&cookie_str, &uri);
            // Add to deleted list
            deleted.insert(name);
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
    /// * `files` - Files to upload as multipart/form-data. Supports:
    ///   - dict[str, str]: field name to file path
    ///   - dict[str, bytes]: field name to file content
    ///   - dict[str, tuple]: field name to (filename, content, mime_type)
    ///   Can be combined with `data` for mixed form fields and files.
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
        files: Option<&Bound<'_, PyAny>>,
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

        // Process files before async block (must be done in Python context)
        enum FileData {
            Path(String, String), // (field_name, file_path)
            Bytes(String, String, Vec<u8>), // (field_name, filename, bytes)
            BytesWithMime(String, String, Vec<u8>, String), // (field_name, filename, bytes, mime)
        }

        let mut files_data: Vec<FileData> = Vec::new();
        if let Some(files_obj) = files {
            if let Ok(files_dict) = files_obj.downcast::<pyo3::types::PyDict>() {
                for (key, value) in files_dict.iter() {
                    let field_name: String = key.extract()?;

                    // Case 1: String (file path)
                    if let Ok(file_path) = value.extract::<String>() {
                        files_data.push(FileData::Path(field_name, file_path));
                    }
                    // Case 2: Bytes (raw data)
                    else if let Ok(bytes) = value.extract::<Vec<u8>>() {
                        files_data.push(FileData::Bytes(field_name.clone(), field_name, bytes));
                    }
                    // Case 3: Tuple (filename, data, [mime_type])
                    else if let Ok(tuple) = value.downcast::<pyo3::types::PyTuple>() {
                        let len = tuple.len();
                        if len >= 2 {
                            let filename: String = tuple.get_item(0)?.extract()?;

                            // Data can be bytes or string (path)
                            if let Ok(bytes) = tuple.get_item(1)?.extract::<Vec<u8>>() {
                                if len >= 3 {
                                    if let Ok(mime_str) = tuple.get_item(2)?.extract::<String>() {
                                        files_data.push(FileData::BytesWithMime(
                                            field_name.clone(),
                                            filename,
                                            bytes,
                                            mime_str,
                                        ));
                                    } else {
                                        files_data.push(FileData::Bytes(field_name.clone(), filename, bytes));
                                    }
                                } else {
                                    files_data.push(FileData::Bytes(field_name.clone(), filename, bytes));
                                }
                            } else if let Ok(path) = tuple.get_item(1)?.extract::<String>() {
                                files_data.push(FileData::Path(field_name, path));
                            }
                        }
                    }
                }
            }
        }

        let has_files = !files_data.is_empty();

        let future = async {
            // Create request builder
            let mut request_builder = client.lock().unwrap().request(method, url);

            // Params
            if let Some(params) = params {
                request_builder = request_builder.query(&params);
            }

            // Calculate body content and length for POST/PUT/PATCH (before setting headers)
            let (body_bytes, content_type_header): (Option<Vec<u8>>, Option<String>) = if is_post_put_patch {
                if has_files {
                    // Multipart will be handled later, can't pre-calculate
                    (None, None)
                } else if let Some(content) = &content {
                    // Raw bytes content
                    (Some(content.clone()), None)
                } else if let Some(form_data) = &data_value {
                    // Data - smart handling
                    if let Some(json_str) = form_data.as_str() {
                        // JSON string
                        if let Ok(parsed_json) = serde_json::from_str::<Value>(json_str) {
                            let serialized = serde_json::to_vec(&parsed_json)?;
                            (Some(serialized), Some("application/json".to_string()))
                        } else {
                            (Some(json_str.as_bytes().to_vec()), None)
                        }
                    } else {
                        // Check if nested
                        let is_nested = if let Some(obj) = form_data.as_object() {
                            obj.values().any(|v| v.is_object() || v.is_array())
                        } else {
                            false
                        };

                        if is_nested {
                            // Nested - use JSON
                            let serialized = serde_json::to_vec(&form_data)?;
                            (Some(serialized), Some("application/json".to_string()))
                        } else {
                            // Flat - use form-urlencoded
                            let encoded = serde_urlencoded::to_string(&form_data)?;
                            (Some(encoded.as_bytes().to_vec()), Some("application/x-www-form-urlencoded".to_string()))
                        }
                    }
                } else if let Some(json_data) = &json_value {
                    // JSON
                    let serialized = serde_json::to_vec(&json_data)?;
                    (Some(serialized), Some("application/json".to_string()))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            // Cookies - get effective cookies (from parameter or cookie_jar)
            // Do this BEFORE processing headers so we can include cookies in header ordering
            let effective_cookies = if let Some(cookies) = cookies {
                Some(cookies)
            } else {
                // Get cookies from cookie_jar
                let jar_cookies = self.get_all_cookies().ok();
                jar_cookies.filter(|c| !c.is_empty())
            };

            // Headers - reorder to match browser behavior: Host first, then Content-Length, then others
            // Check both request-level and client-level ordered_headers
            let effective_ordered_headers = if ordered_headers.is_some() {
                ordered_headers
            } else {
                // Use client-level ordered_headers if no request-level specified
                self.ordered_headers.lock().ok().and_then(|guard| guard.clone())
            };

            if let Some(ordered_hdrs) = effective_ordered_headers {
                // Create a new ordered map with strict ordering
                let mut reordered_headers = IndexMap::with_capacity_and_hasher(ordered_hdrs.len() + 2, RandomState::default());

                // 1. First, add Host header if present (case-insensitive check)
                let host_value = ordered_hdrs.get("Host")
                    .or_else(|| ordered_hdrs.get("host"))
                    .or_else(|| ordered_hdrs.get("HOST"));

                if let Some(host) = host_value {
                    reordered_headers.insert("Host".to_string(), host.clone());
                }

                // 2. For POST/PUT/PATCH with body, add Content-Length in 2nd position
                if let Some(ref body) = body_bytes {
                    let content_length = body.len().to_string();
                    reordered_headers.insert("Content-Length".to_string(), content_length);
                } else if has_files {
                    // For multipart, we can't pre-calculate, but reserve the position
                    // This will be overwritten by wreq, but maintains position
                    reordered_headers.insert("Content-Length".to_string(), "0".to_string());
                }

                // 3. Add Content-Type if we calculated it (and user didn't specify)
                if let Some(ct) = content_type_header {
                    let has_content_type = ordered_hdrs.iter().any(|(k, _)| k.to_lowercase() == "content-type");
                    if !has_content_type {
                        reordered_headers.insert("Content-Type".to_string(), ct);
                    }
                }

                // 4. Add all other headers in their original order (skip Host, Content-Length, Content-Type, priority, cookie)
                // priority and cookie will be added at the end
                let mut priority_header: Option<(String, String)> = None;
                let mut cookie_from_headers: Option<(String, String)> = None;

                for (key, value) in ordered_hdrs.iter() {
                    let key_lower = key.to_lowercase();
                    // Skip if already added or if it's priority/cookie (will be added later)
                    if key_lower == "host" || key_lower == "content-length" || reordered_headers.contains_key(key) {
                        continue;
                    }
                    if key_lower == "priority" {
                        priority_header = Some((key.clone(), value.clone()));
                    } else if key_lower == "cookie" {
                        cookie_from_headers = Some((key.clone(), value.clone()));
                    } else {
                        reordered_headers.insert(key.clone(), value.clone());
                    }
                }

                // 5. Handle cookies based on split_cookies option
                let should_add_cookies_separately = self.split_cookies.unwrap_or(false);

                // Build orig_headermap manually to control exact order
                let mut orig_headermap = OrigHeaderMap::with_capacity(reordered_headers.len() + 10);

                // Add all current headers to orig_headermap
                for (key, _) in reordered_headers.iter() {
                    orig_headermap.insert(key.clone());
                }

                if should_add_cookies_separately {
                    // Split cookies: add each cookie as a separate header in orig_headermap
                    if let Some(cookies) = &effective_cookies {
                        for (_k, _v) in cookies.iter() {
                            // Add to orig_headermap for ordering
                            orig_headermap.insert("cookie".to_string());
                            // Add to request_builder after applying headers
                        }
                    } else if let Some((_, ref value)) = cookie_from_headers {
                        // Split the cookie value and add each part
                        for part in value.split(';') {
                            let part = part.trim();
                            if !part.is_empty() {
                                orig_headermap.insert("cookie".to_string());
                            }
                        }
                    }

                    // Add priority to orig_headermap at the end
                    if let Some((ref key, _)) = priority_header {
                        orig_headermap.insert(key.clone());
                    }
                } else {
                    // Merge cookies into single header
                    if let Some(cookies) = &effective_cookies {
                        if !cookies.is_empty() {
                            let cookie_value = cookies
                                .iter()
                                .map(|(k, v)| format!("{}={}", k, v))
                                .collect::<Vec<_>>()
                                .join("; ");
                            reordered_headers.insert("cookie".to_string(), cookie_value);
                            orig_headermap.insert("cookie".to_string());
                        }
                    } else if let Some((ref key, ref value)) = cookie_from_headers {
                        reordered_headers.insert(key.clone(), value.clone());
                        orig_headermap.insert(key.clone());
                    }

                    // Add priority at the very end
                    if let Some((ref key, ref value)) = priority_header {
                        reordered_headers.insert(key.clone(), value.clone());
                        orig_headermap.insert(key.clone());
                    }
                }

                // Apply the reordered headers with strict order preservation
                let headers_headermap = reordered_headers.to_headermap();
                request_builder = request_builder
                    .headers(headers_headermap)
                    .orig_headers(orig_headermap);

                // If split_cookies=true, add cookies separately using header_append
                if should_add_cookies_separately {
                    if let Some(cookies) = &effective_cookies {
                        if !cookies.is_empty() {
                            for (k, v) in cookies.iter() {
                                let cookie_value = format!("{}={}", k, v);
                                request_builder = request_builder.header_append("cookie", cookie_value);
                            }
                        }
                    } else if let Some((_, ref value)) = cookie_from_headers {
                        // If cookie came from ordered_headers, split it
                        for part in value.split(';') {
                            let part = part.trim();
                            if !part.is_empty() {
                                request_builder = request_builder.header_append("cookie", part);
                            }
                        }
                    }

                    // Add priority after cookies to maintain order
                    // Use header_append to ensure it's added at the end
                    if let Some((ref key, ref value)) = priority_header {
                        request_builder = request_builder.header_append(key, value);
                    }
                }
            } else if let Some(headers) = headers {
                // Fallback to regular headers
                request_builder = request_builder.headers(headers.to_headermap());

                // Add cookies separately if using regular headers
                if let Some(cookies) = &effective_cookies {
                    if !cookies.is_empty() {
                        if self.split_cookies.unwrap_or(false) {
                            // Split: multiple cookie headers (HTTP/2 style)
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
            }

            // Only if method POST || PUT || PATCH
            if is_post_put_patch {
                // Files - handle multipart/form-data
                if has_files {
                    let mut form = multipart::Form::new();

                    // Add data fields to multipart if present
                    if let Some(form_data) = &data_value {
                        if let Some(obj) = form_data.as_object() {
                            for (key, value) in obj {
                                let value_str = match value {
                                    Value::String(s) => s.clone(),
                                    _ => value.to_string(),
                                };
                                form = form.text(key.clone(), value_str);
                            }
                        }
                    }

                    // Process files
                    for file_data in files_data {
                        match file_data {
                            FileData::Path(field_name, file_path) => {
                                let file = File::open(&file_path).await?;
                                let stream = FramedRead::new(file, BytesCodec::new());
                                let file_body = Body::wrap_stream(stream);

                                // Extract filename from path
                                let filename = std::path::Path::new(&file_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or(&field_name)
                                    .to_string();

                                let part = multipart::Part::stream(file_body).file_name(filename);
                                form = form.part(field_name, part);
                            }
                            FileData::Bytes(field_name, filename, bytes) => {
                                let part = multipart::Part::bytes(bytes).file_name(filename);
                                form = form.part(field_name, part);
                            }
                            FileData::BytesWithMime(field_name, filename, bytes, mime_str) => {
                                let mut part = multipart::Part::bytes(bytes).file_name(filename);
                                if let Ok(mime) = mime_str.parse::<mime::Mime>() {
                                    part = part.mime_str(mime.as_ref())?;
                                }
                                form = form.part(field_name, part);
                            }
                        }
                    }

                    request_builder = request_builder.multipart(form);
                }
                // Use pre-serialized body bytes
                else if let Some(body) = body_bytes {
                    request_builder = request_builder.body(body);
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
            if let Some(_ordered_hdrs) = ordered_guard.as_ref() {
                // Don't set ordered_headers as default_headers to avoid duplication
                // They will be applied dynamically at request time with proper ordering
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

        // Http1_only and Http2_only (mutually exclusive, http1_only takes precedence)
        if let Some(true) = self.http1_only {
            client_builder = client_builder.http1_only();
        } else if let Some(true) = self.http2_only {
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
