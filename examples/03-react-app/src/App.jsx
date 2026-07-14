export default function App({ name = "World" }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <title>Klyron + React SSR</title>
        <link
          rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"
        />
      </head>
      <body>
        <main className="container">
          <h1>Hello, {name}!</h1>
          <p>
            This page was rendered on the server by{" "}
            <strong>Klyron</strong> using <strong>React</strong> SSR.
          </p>
          <p>The time is {new Date().toLocaleTimeString()}.</p>
        </main>
      </body>
    </html>
  );
}
