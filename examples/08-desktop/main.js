const port = 3000;

const notes = [
  { id: 1, title: "Welcome", content: "Welcome to Klyron Desktop!" },
  { id: 2, title: "Getting Started", content: "Write your notes here." },
];

function readFile(pathname) {
  const filePath = pathname === "/" ? "/public/index.html" : `/public${pathname}`;
  try {
    return Deno.readTextFileSync(new URL(filePath, import.meta.url).pathname);
  } catch {
    return null;
  }
}

Klyron.serve({ port }, async (req) => {
  const url = new URL(req.url);
  const { pathname, method } = url;

  // API routes
  if (pathname === "/api/notes" && method === "GET") {
    return new Response(JSON.stringify(notes), {
      headers: { "Content-Type": "application/json" },
    });
  }

  if (pathname === "/api/notes" && method === "POST") {
    const body = await req.json();
    const note = { id: notes.length + 1, title: body.title, content: body.content };
    notes.push(note);
    return new Response(JSON.stringify(note), {
      status: 201,
      headers: { "Content-Type": "application/json" },
    });
  }

  if (pathname.startsWith("/api/notes/") && method === "DELETE") {
    const id = parseInt(pathname.split("/")[3], 10);
    const idx = notes.findIndex((n) => n.id === id);
    if (idx !== -1) {
      notes.splice(idx, 1);
      return new Response(JSON.stringify({ ok: true }), {
        headers: { "Content-Type": "application/json" },
      });
    }
    return new Response(JSON.stringify({ error: "not found" }), {
      status: 404,
      headers: { "Content-Type": "application/json" },
    });
  }

  // Static files
  const ext = pathname.split(".").pop();
  const mimeTypes = {
    html: "text/html",
    js: "application/javascript",
    css: "text/css",
  };
  const content = readFile(pathname);
  if (content) {
    return new Response(content, {
      headers: { "Content-Type": mimeTypes[ext] || "text/plain" },
    });
  }

  return new Response("Not Found", { status: 404 });
});

console.log(`Desktop app running at http://localhost:${port}`);
