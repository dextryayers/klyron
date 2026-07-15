use std::sync::{Arc, Mutex};

use deno_core::{extension, op2, Extension, OpState};
use deno_error::JsErrorBox;

type ServerHandle = Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>;

extension!(
  klyron_http,
  ops = [op_http_serve, op_http_stop, op_http_json, op_http_html, op_http_text, op_http_redirect],
  esm_entry_point = "ext:klyron_http/http.js",
  esm = [dir "js", "http.js"],
  state = |state| { state.put::<ServerHandle>(Arc::new(Mutex::new(None))); },
);

pub fn init() -> Extension {
  klyron_http::init()
}

#[op2]
#[string]
fn op_http_serve(state: &mut OpState, #[string] addr: String) -> Result<String, JsErrorBox> {
  let bind_addr = addr.clone();
  let rt = tokio::runtime::Handle::current();
  let handle = rt.spawn(async move {
    let listener = match tokio::net::TcpListener::bind(&bind_addr).await {
      Ok(l) => l,
      Err(e) => { eprintln!("HTTP serve bind error: {e}"); return; }
    };
    loop {
      match listener.accept().await {
        Ok((mut stream, _)) => {
          tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let n = match tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await {
              Ok(n) if n > 0 => n,
              _ => return,
            };

            let request_text = String::from_utf8_lossy(&buf[..n]);
            let mut lines = request_text.lines();
            let request_line = lines.next().unwrap_or("");
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            let method = parts.first().unwrap_or(&"GET");
            let path = parts.get(1).unwrap_or(&"/");

            let mut body_start = 0;
            let mut content_length = 0usize;
            for line in lines.by_ref() {
              if line.is_empty() {
                body_start = line.as_ptr() as usize - buf.as_ptr() as usize + 2;
                break;
              }
              if let Some(cl) = line.to_lowercase().strip_prefix("content-length:") {
                content_length = cl.trim().parse().unwrap_or(0);
              }
            }

            let body = if content_length > 0 && body_start > 0 && body_start + content_length <= n {
              String::from_utf8_lossy(&buf[body_start..body_start + content_length]).to_string()
            } else {
              String::new()
            };

            let status_line = "HTTP/1.1 200 OK\r\n";
            let response_body = format!("{{\"method\":\"{}\",\"path\":\"{}\",\"body\":{}}}",
              method, path,
              if body.is_empty() { "null".to_string() } else { serde_json::to_string(&body).unwrap_or_default() }
            );
            let response = format!("{}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
              status_line, response_body.len(), response_body);

            let _ = tokio::io::AsyncWriteExt::write_all(&mut stream, response.as_bytes()).await;
          });
        }
        Err(_) => break,
      }
    }
  });

  let server_handle = state.borrow_mut::<ServerHandle>();
  *server_handle.lock().unwrap() = Some(handle);
  Ok(format!("Serving on http://{addr}"))
}

#[op2(fast)]
fn op_http_stop(state: &mut OpState) -> Result<(), JsErrorBox> {
  let server_handle = state.borrow_mut::<ServerHandle>();
  if let Some(handle) = server_handle.lock().unwrap().take() {
    handle.abort();
  }
  Ok(())
}

#[op2]
#[string]
fn op_http_json(#[string] data: String, status: f64) -> String {
  let code = status as u16;
  let body = data;
  let status_line = match code {
    200 => "HTTP/1.1 200 OK",
    201 => "HTTP/1.1 201 Created",
    204 => "HTTP/1.1 204 No Content",
    400 => "HTTP/1.1 400 Bad Request",
    401 => "HTTP/1.1 401 Unauthorized",
    403 => "HTTP/1.1 403 Forbidden",
    404 => "HTTP/1.1 404 Not Found",
    405 => "HTTP/1.1 405 Method Not Allowed",
    409 => "HTTP/1.1 409 Conflict",
    422 => "HTTP/1.1 422 Unprocessable Entity",
    429 => "HTTP/1.1 429 Too Many Requests",
    500 => "HTTP/1.1 500 Internal Server Error",
    502 => "HTTP/1.1 502 Bad Gateway",
    503 => "HTTP/1.1 503 Service Unavailable",
    _ => "HTTP/1.1 200 OK",
  };
  format!(
    "{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
    status_line,
    body.len(),
    body
  )
}

#[op2]
#[string]
fn op_http_html(#[string] data: String, status: f64) -> String {
  let code = status as u16;
  let status_line = match code {
    200 => "HTTP/1.1 200 OK",
    201 => "HTTP/1.1 201 Created",
    404 => "HTTP/1.1 404 Not Found",
    500 => "HTTP/1.1 500 Internal Server Error",
    _ => "HTTP/1.1 200 OK",
  };
  format!(
    "{}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
    status_line,
    data.len(),
    data
  )
}

#[op2]
#[string]
fn op_http_text(#[string] data: String, status: f64) -> String {
  let code = status as u16;
  let status_line = match code {
    200 => "HTTP/1.1 200 OK",
    201 => "HTTP/1.1 201 Created",
    400 => "HTTP/1.1 400 Bad Request",
    404 => "HTTP/1.1 404 Not Found",
    500 => "HTTP/1.1 500 Internal Server Error",
    _ => "HTTP/1.1 200 OK",
  };
  format!(
    "{}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
    status_line,
    data.len(),
    data
  )
}

#[op2]
#[string]
fn op_http_redirect(#[string] location: String, status: f64) -> String {
  let code = status as u16;
  let status_line = match code {
    301 => "HTTP/1.1 301 Moved Permanently",
    302 => "HTTP/1.1 302 Found",
    303 => "HTTP/1.1 303 See Other",
    307 => "HTTP/1.1 307 Temporary Redirect",
    308 => "HTTP/1.1 308 Permanent Redirect",
    _ => "HTTP/1.1 302 Found",
  };
  format!(
    "{}\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    status_line,
    location
  )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_http");
    }

    #[test]
    fn test_http_json_200() {
        let result = op_http_json(r#"{"message":"ok"}"#.to_string(), 200.0);
        assert!(result.starts_with("HTTP/1.1 200 OK"));
        assert!(result.contains("Content-Type: application/json"));
        assert!(result.contains(r#"{"message":"ok"}"#));
    }

    #[test]
    fn test_http_json_404() {
        let result = op_http_json(r#"{"error":"not found"}"#.to_string(), 404.0);
        assert!(result.starts_with("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn test_http_json_500() {
        let result = op_http_json(r#"{"error":"server error"}"#.to_string(), 500.0);
        assert!(result.starts_with("HTTP/1.1 500 Internal Server Error"));
    }

    #[test]
    fn test_http_html_200() {
        let result = op_http_html("<h1>Hello</h1>".to_string(), 200.0);
        assert!(result.starts_with("HTTP/1.1 200 OK"));
        assert!(result.contains("Content-Type: text/html"));
    }

    #[test]
    fn test_http_html_404() {
        let result = op_http_html("not found".to_string(), 404.0);
        assert!(result.starts_with("HTTP/1.1 404 Not Found"));
    }

    #[test]
    fn test_http_text_200() {
        let result = op_http_text("plain text".to_string(), 200.0);
        assert!(result.starts_with("HTTP/1.1 200 OK"));
        assert!(result.contains("Content-Type: text/plain"));
    }

    #[test]
    fn test_http_text_400() {
        let result = op_http_text("bad request".to_string(), 400.0);
        assert!(result.starts_with("HTTP/1.1 400 Bad Request"));
    }

    #[test]
    fn test_http_redirect_301() {
        let result = op_http_redirect("/new-location".to_string(), 301.0);
        assert!(result.starts_with("HTTP/1.1 301 Moved Permanently"));
        assert!(result.contains("Location: /new-location"));
    }

    #[test]
    fn test_http_redirect_302() {
        let result = op_http_redirect("/temp".to_string(), 302.0);
        assert!(result.starts_with("HTTP/1.1 302 Found"));
    }

    #[test]
    fn test_http_redirect_308() {
        let result = op_http_redirect("/perm".to_string(), 308.0);
        assert!(result.starts_with("HTTP/1.1 308 Permanent Redirect"));
    }

    #[test]
    fn test_http_json_201() {
        let result = op_http_json(r#"{"created":true}"#.to_string(), 201.0);
        assert!(result.starts_with("HTTP/1.1 201 Created"));
    }

    #[test]
    fn test_http_text_content_length() {
        let result = op_http_text("hello".to_string(), 200.0);
        assert!(result.contains("Content-Length: 5"));
    }

    #[test]
    fn test_http_json_content_length() {
        let body = r#"{"x":1}"#;
        let result = op_http_json(body.to_string(), 200.0);
        assert!(result.contains(&format!("Content-Length: {}", body.len())));
    }

    #[test]
    fn test_http_json_unauthorized() {
        let result = op_http_json("error".to_string(), 401.0);
        assert!(result.starts_with("HTTP/1.1 401 Unauthorized"));
    }

    #[test]
    fn test_http_text_forbidden() {
        let result = op_http_text("forbidden".to_string(), 403.0);
        assert!(result.starts_with("HTTP/1.1 403 Forbidden"));
    }

    #[test]
    fn test_http_json_conflict() {
        let result = op_http_json("conflict".to_string(), 409.0);
        assert!(result.starts_with("HTTP/1.1 409 Conflict"));
    }

    #[test]
    fn test_http_json_422() {
        let result = op_http_json("error".to_string(), 422.0);
        assert!(result.starts_with("HTTP/1.1 422 Unprocessable Entity"));
    }

    #[test]
    fn test_http_json_429() {
        let result = op_http_json("rate limited".to_string(), 429.0);
        assert!(result.starts_with("HTTP/1.1 429 Too Many Requests"));
    }

    #[test]
    fn test_http_html_201() {
        let result = op_http_html("<p>created</p>".to_string(), 201.0);
        assert!(result.starts_with("HTTP/1.1 201 Created"));
    }

    #[test]
    fn test_http_redirect_303() {
        let result = op_http_redirect("/other".to_string(), 303.0);
        assert!(result.starts_with("HTTP/1.1 303 See Other"));
    }

    #[test]
    fn test_http_redirect_307() {
        let result = op_http_redirect("/temp".to_string(), 307.0);
        assert!(result.starts_with("HTTP/1.1 307 Temporary Redirect"));
    }
}
