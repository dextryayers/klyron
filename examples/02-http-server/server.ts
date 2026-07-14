const port: number = 3000;

Klyron.serve({ port }, (req: Request): Response => {
  const url = new URL(req.url);
  const name = url.searchParams.get("name") || "World";

  if (url.pathname === "/") {
    return new Response(indexHtml(name), {
      headers: { "Content-Type": "text/html" },
    });
  }

  if (url.pathname === "/api/hello") {
    return new Response(
      JSON.stringify({ message: `Hello, ${name}!`, lang: "TypeScript" }),
      {
        headers: { "Content-Type": "application/json" },
      },
    );
  }

  return new Response("Not Found", { status: 404 });
});

function indexHtml(name: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Klyron HTTP Server</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css" />
</head>
<body>
  <main class="container">
    <h1>Hello, ${name}!</h1>
    <p>This page is served by <strong>Klyron</strong> using <strong>TypeScript</strong>.</p>
    <p><a href="/api/hello">Check the JSON API</a></p>
  </main>
</body>
</html>`;
}

console.log(`Server running at http://localhost:${port}`);
