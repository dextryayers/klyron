// klyron Bun compatibility shim.
// Provides globalThis.Bun with `serve` and `file`, layered on top of the
// node http server and the klyron fs operations.
import http from "ext:klyron_node/http.js";
import fs from "ext:klyron_node/fs.js";
import cp from "ext:klyron_node/child_process.js";
import { Headers as WebHeaders, Request as WebRequest, Response as WebResponse } from "ext:klyron_web/web.js";

function nodeHeadersToWeb(headers) {
  const h = new WebHeaders();
  for (const k in headers) {
    if (Object.prototype.hasOwnProperty.call(headers, k)) {
      h.set(k, headers[k]);
    }
  }
  return h;
}

function collectBody(req) {
  return new Promise((resolve) => {
    const chunks = [];
    req.on("data", (c) => chunks.push(c));
    req.on("end", () => {
      let buf = Buffer.alloc(0);
      for (const c of chunks) buf = Buffer.concat([buf, c]);
      resolve(buf);
    });
  });
}

function webResponseToNode(res, webRes) {
  res.statusCode = webRes.status;
  webRes.headers.forEach((v, k) => res.setHeader(k, v));
  return webRes.text().then((t) => {
    res.end(Buffer.from(String(t)));
  });
}

const Bun = {
  // Bun.serve({ port, hostname, fetch }) -> server handle with .stop()
  serve(options) {
    const opts = options || {};
    const port = opts.port ?? 3000;
    const hostname = opts.hostname ?? opts.host ?? "0.0.0.0";
    const fetchHandler = opts.fetch;

    const server = http.createServer((req, res) => {
      const url = "http://" + (req.headers.host || hostname + ":" + port) + req.url;
      const webReq = new WebRequest(url, {
        method: req.method,
        headers: nodeHeadersToWeb(req.headers),
      });

      const handle = () => {
        let result;
        try {
          result = fetchHandler(webReq);
        } catch (err) {
          res.statusCode = 500;
          res.end("Internal Server Error");
          return;
        }
        Promise.resolve(result)
          .then((webRes) => {
            if (!(webRes instanceof WebResponse)) {
              res.statusCode = 200;
              res.end(Buffer.from(String(webRes)));
              return;
            }
            return webResponseToNode(res, webRes);
          })
          .catch((err) => {
            if (!res.headersSent) {
              res.statusCode = 500;
              res.end("Internal Server Error");
            }
          });
      };

      collectBody(req).then((buf) => {
        try {
          Object.defineProperty(webReq, "arrayBuffer", {
            value: () => Promise.resolve(buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength)),
            configurable: true,
          });
          Object.defineProperty(webReq, "text", {
            value: () => Promise.resolve(buf.toString("utf8")),
            configurable: true,
          });
        } catch (_) {}
        handle();
      });
    });

    let listening = false;
    const handle = {
      get stopped() { return !listening; },
      stop() {
        return new Promise((resolve) => {
          server.close(() => {
            listening = false;
            resolve();
          });
        });
      },
      reload() {},
    };

    server.listen(port, hostname, () => {
      listening = true;
    });

    return handle;
  },

  // Bun.file(path) -> BunFile-like object.
  file(path) {
    const p = String(path);
    return {
      get name() {
        const parts = p.split(/[\\/]/);
        return parts[parts.length - 1] || p;
      },
      get path() { return p; },
      async text() {
        return fs.readFileSync(p, "utf8");
      },
      async arrayBuffer() {
        const s = fs.readFileSync(p, "utf8");
        const buf = Buffer.from(s, "utf8");
        return buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength);
      },
      async json() {
        return JSON.parse(fs.readFileSync(p, "utf8"));
      },
      async write(data) {
        fs.writeFileSync(p, String(data), "utf8");
        return this;
      },
      async exists() {
        try {
          fs.readFileSync(p, "utf8");
          return true;
        } catch (_) {
          return false;
        }
      },
    };
  },

  // Bun.spawn is a minimal wrapper exposing process spawning.
  spawn(cmd, opts) {
    return cp.spawn(cmd, opts);
  },
};

globalThis.Bun = Bun;

export default Bun;
export { Bun };
