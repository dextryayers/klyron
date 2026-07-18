use crate::error::V8Error;

#[cfg(feature = "native")]
use crate::ffi;
#[cfg(feature = "native")]
use std::ffi::CString;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Url {
    pub href: String,
    pub protocol: String,
    pub hostname: String,
    pub port: String,
    pub pathname: String,
    pub search: String,
    pub hash: String,
    pub host: String,
    pub origin: String,
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.href)
    }
}

pub fn parse_url(url: &str, base: Option<&str>) -> Result<Url, V8Error> {
    #[cfg(feature = "native")]
    {
        let c_url = CString::new(url).map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let c_base = base.map(|b| CString::new(b)).transpose().map_err(|e| V8Error::EvalFailed(e.to_string()))?;
        let ptr = unsafe {
            ffi::klyron_v8_url_parse(c_url.as_ptr(), c_base.as_ref().map_or(std::ptr::null(), |b| b.as_ptr()))
        };
        if ptr.is_null() {
            return Err(V8Error::EvalFailed("URL parse failed".into()));
        }
        let url = unsafe {
            let href = if (*ptr).href.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).href).to_string_lossy().into() };
            let protocol = if (*ptr).protocol.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).protocol).to_string_lossy().into() };
            let hostname = if (*ptr).hostname.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).hostname).to_string_lossy().into() };
            let port = if (*ptr).port.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).port).to_string_lossy().into() };
            let pathname = if (*ptr).pathname.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).pathname).to_string_lossy().into() };
            let search = if (*ptr).search.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).search).to_string_lossy().into() };
            let hash = if (*ptr).hash.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).hash).to_string_lossy().into() };
            let host = if (*ptr).host.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).host).to_string_lossy().into() };
            let origin = if (*ptr).origin.is_null() { String::new() } else { std::ffi::CStr::from_ptr((*ptr).origin).to_string_lossy().into() };
            Url { href, protocol, hostname, port, pathname, search, hash, host, origin }
        };
        unsafe { ffi::klyron_v8_url_dispose(ptr) };
        Ok(url)
    }

    #[cfg(not(feature = "native"))]
    {
        let _ = (url, base);
        Err(V8Error::NotInitialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_parse_simple() {
        if cfg!(not(feature = "native")) { return; }
        let u = parse_url("https://example.com/path", None);
        assert!(u.is_ok());
        if let Ok(url) = u {
            assert_eq!(url.protocol, "https:");
            assert_eq!(url.hostname, "example.com");
            assert_eq!(url.pathname, "/path");
        }
    }
}
