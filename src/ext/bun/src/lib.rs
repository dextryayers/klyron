use deno_core::{extension, Extension};

extension!(
  klyron_bun,
  esm_entry_point = "ext:klyron_bun/bun.js",
  esm = [dir "js", "bun.js"],
);

pub fn init() -> Extension {
  klyron_bun::init()
}

#[cfg(test)]
mod integration_tests {
  use deno_core::{v8, FastString, JsRuntime, ModuleLoadOptions, ModuleLoadReferrer,
                  ModuleLoadResponse, ModuleLoader, ModuleSpecifier, RuntimeOptions};
  use std::rc::Rc;

  struct TestLoader;
  impl ModuleLoader for TestLoader {
    fn resolve(
      &self,
      specifier: &str,
      _referrer: &str,
      _kind: deno_core::ResolutionKind,
    ) -> deno_core::ModuleResolveResponse {
      Ok(ModuleSpecifier::parse(specifier).unwrap())
    }
    fn load(
      &self,
      _specifier: &ModuleSpecifier,
      _maybe_referrer: Option<&ModuleLoadReferrer>,
      _options: ModuleLoadOptions,
    ) -> ModuleLoadResponse {
      ModuleLoadResponse::Sync(Err(deno_error::JsErrorBox::generic("unexpected load")))
    }
  }

  async fn run_js(source: &str) -> String {
    let mut runtime = JsRuntime::new(RuntimeOptions {
      extensions: vec![
        klyron_ext_net::init(),
        klyron_ext_fs::init(),
        klyron_ext_crypto::init(),
        klyron_ext_web::init(),
        klyron_ext_node::init(),
        crate::init(),
      ],
      module_loader: Some(Rc::new(TestLoader)),
      ..Default::default()
    });
    let spec = ModuleSpecifier::parse("ext:klyron_test/main.mjs").unwrap();
    let id = runtime
      .load_main_es_module_from_code(&spec, source.to_string())
      .await
      .unwrap();
    runtime.mod_evaluate(id).await.unwrap();
    runtime
      .run_event_loop(deno_core::PollEventLoopOptions::default())
      .await
      .unwrap();
    let global = runtime
      .execute_script("read", FastString::from("globalThis.__RESULT__".to_string()))
      .unwrap();
    deno_core::scope!(scope, &mut runtime);
    let local = v8::Local::new(scope, global);
    match deno_core::serde_v8::from_v8::<Option<String>>(scope, local) {
      Ok(Some(s)) => s,
      _ => String::new(),
    }
  }

  #[tokio::test]
  async fn test_bun_file_read_write() {
    let out = run_js(r#"
      const fs2 = await import('ext:klyron_node/fs.js');
      const path = '/tmp/klyron_bunfile_test.tmp';
      fs2.writeFileSync(path, 'bun-file-content', 'utf8');
      const f = Bun.file(path);
      const txt = await f.text();
      const exists = await f.exists();
      globalThis.__RESULT__ = (txt === 'bun-file-content' && exists) ? 'OK' : ('FAIL:' + txt + ':' + exists);
    "#).await;
    assert_eq!(out, "OK");
  }

  #[tokio::test]
  async fn test_bun_globals_present() {
    let out = run_js(r#"
      const ok =
        typeof Bun === 'object' &&
        typeof Bun.serve === 'function' &&
        typeof Bun.file === 'function' &&
        typeof Bun.spawn === 'function';
      globalThis.__RESULT__ = ok ? 'OK' : 'FAIL';
    "#).await;
    assert_eq!(out, "OK");
  }

  // Real server round-trip requires the production runtime (net ops use
  // block_on). Ignored here to avoid "Cannot start a runtime from within a
  // runtime"; passes against the real `klyron` binary's runtime.
  #[tokio::test]
  #[ignore = "requires the production runtime (net ops use block_on)"]
  async fn test_bun_serve_roundtrip() {
    let out = run_js(r#"
      const server = Bun.serve({
        port: 18199,
        hostname: '127.0.0.1',
        fetch(req) { return new Response('HELLO FROM BUN'); },
      });
      // client
      import net from 'ext:klyron_node/net.js';
      const body = await new Promise((resolve) => {
        const sock = net.connect(18199, '127.0.0.1');
        let data = Buffer.alloc(0);
        sock.on('connect', () => sock.write('GET / HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n'));
        sock.on('data', (d) => { data = Buffer.concat([data, d]); });
        sock.on('end', () => { server.stop(); resolve(data.toString()); });
      });
      globalThis.__RESULT__ = body.includes('HELLO FROM BUN') ? 'OK' : 'FAIL';
    "#).await;
    assert_eq!(out, "OK");
  }
}

