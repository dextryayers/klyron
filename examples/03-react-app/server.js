import React from "react";
import { renderToString } from "react-dom/server";
import App from "./src/App.jsx";

const port = 3000;
const htmlTemplate = await Deno.readTextFile(
  new URL("./src/index.html", import.meta.url).pathname
);

Klyron.serve({ port }, (req) => {
  const url = new URL(req.url);
  const name = url.searchParams.get("name") || "Klyron";

  if (url.pathname === "/") {
    const appHtml = renderToString(React.createElement(App, { name }));
    const html = htmlTemplate.replace("<!--SSR_CONTENT-->", appHtml);
    return new Response(html, {
      headers: { "Content-Type": "text/html" },
    });
  }

  return new Response("Not Found", { status: 404 });
});

console.log(`React SSR server running at http://localhost:${port}`);
