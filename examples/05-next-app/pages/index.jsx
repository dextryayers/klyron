import Head from "next/head";

export default function Home() {
  return (
    <>
      <Head>
        <title>Klyron + Next.js</title>
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <link
          rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/@picocss/pico@2/css/pico.min.css"
        />
      </Head>
      <main className="container">
        <h1>Hello, Klyron!</h1>
        <p>
          This page is served by <strong>Klyron</strong> using{" "}
          <strong>Next.js</strong> with <strong>React</strong>.
        </p>
        <p>
          <a href="/api/hello">Check the API route</a>
        </p>
      </main>
    </>
  );
}

export async function getServerSideProps() {
  return {
    props: {
      renderedAt: new Date().toISOString(),
    },
  };
}
