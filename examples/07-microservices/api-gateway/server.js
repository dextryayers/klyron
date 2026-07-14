const port = 8080;
const USERS_SERVICE = Deno.env.get("USERS_SERVICE_URL") || "http://localhost:4001";
const ORDERS_SERVICE = Deno.env.get("ORDERS_SERVICE_URL") || "http://localhost:4002";

async function proxy(url, baseUrl) {
  try {
    const res = await fetch(`${baseUrl}${url.pathname}`, {
      method: "GET",
      headers: { "Content-Type": "application/json" },
    });
    const data = await res.json();
    return new Response(JSON.stringify(data), {
      status: res.status,
      headers: { "Content-Type": "application/json" },
    });
  } catch (err) {
    return new Response(
      JSON.stringify({ error: "service unavailable", detail: err.message }),
      { status: 503, headers: { "Content-Type": "application/json" } }
    );
  }
}

Klyron.serve({ port }, (req) => {
  const url = new URL(req.url);

  if (url.pathname.startsWith("/api/users")) {
    return proxy(url, USERS_SERVICE);
  }

  if (url.pathname.startsWith("/api/orders")) {
    return proxy(url, ORDERS_SERVICE);
  }

  if (url.pathname === "/api/health") {
    return new Response(
      JSON.stringify({ status: "ok", service: "api-gateway" }),
      { headers: { "Content-Type": "application/json" } }
    );
  }

  if (url.pathname === "/") {
    return new Response(
      `<h1>Klyron API Gateway</h1><p>Endpoints: <a href="/api/users">/api/users</a>, <a href="/api/orders">/api/orders</a>, <a href="/api/health">/api/health</a></p>`,
      { headers: { "Content-Type": "text/html" } }
    );
  }

  return new Response("Not Found", { status: 404 });
});

console.log(`API Gateway running on port ${port}`);
console.log(`  Users service -> ${USERS_SERVICE}`);
console.log(`  Orders service -> ${ORDERS_SERVICE}`);
