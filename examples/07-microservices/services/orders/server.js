const port = 4002;

const orders = [
  { id: 1, userId: 1, product: "Widget", amount: 29.99 },
  { id: 2, userId: 1, product: "Gadget", amount: 49.99 },
  { id: 3, userId: 2, product: "Doohickey", amount: 9.99 },
];

Klyron.serve({ port }, (req) => {
  const url = new URL(req.url);

  if (url.pathname === "/orders") {
    return new Response(JSON.stringify(orders), {
      headers: { "Content-Type": "application/json" },
    });
  }

  if (url.pathname.startsWith("/orders/")) {
    const id = parseInt(url.pathname.split("/")[2], 10);
    const order = orders.find((o) => o.id === id);
    if (order) {
      return new Response(JSON.stringify(order), {
        headers: { "Content-Type": "application/json" },
      });
    }
    return new Response(JSON.stringify({ error: "not found" }), {
      status: 404,
      headers: { "Content-Type": "application/json" },
    });
  }

  return new Response("Not Found", { status: 404 });
});

console.log(`Orders service running on port ${port}`);
