const port = 3000;

const tasks = [
  { id: 1, title: "Learn Klyron", done: true },
  { id: 2, title: "Build a full-stack app", done: false },
  { id: 3, title: "Deploy to production", done: false },
];

function serveStatic(pathname) {
  const filePath = pathname === "/" ? "/client/index.html" : `/client${pathname}`;
  try {
    const content = Deno.readTextFileSync(
      new URL(filePath, import.meta.url).pathname
    );
    const ext = filePath.split(".").pop();
    const mimeTypes = {
      html: "text/html",
      js: "application/javascript",
      css: "text/css",
      json: "application/json",
    };
    return new Response(content, {
      headers: { "Content-Type": mimeTypes[ext] || "text/plain" },
    });
  } catch {
    return null;
  }
}

Klyron.serve({ port }, async (req) => {
  const url = new URL(req.url);
  const { pathname } = url;

  // API routes
  if (pathname === "/api/health") {
    return new Response(
      JSON.stringify({ status: "ok", uptime: process.uptime() }),
      { headers: { "Content-Type": "application/json" } }
    );
  }

  if (pathname === "/api/tasks" && req.method === "GET") {
    return new Response(JSON.stringify(tasks), {
      headers: { "Content-Type": "application/json" },
    });
  }

  if (pathname === "/api/tasks" && req.method === "POST") {
    const body = await req.json();
    const task = { id: tasks.length + 1, title: body.title, done: false };
    tasks.push(task);
    return new Response(JSON.stringify(task), {
      status: 201,
      headers: { "Content-Type": "application/json" },
    });
  }

  // Static files
  const staticResponse = serveStatic(pathname);
  if (staticResponse) return staticResponse;

  return new Response("Not Found", { status: 404 });
});

console.log(`Full-stack server running at http://localhost:${port}`);
