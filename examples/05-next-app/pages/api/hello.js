export default function handler(req, res) {
  const { name = "Klyron" } = req.query;

  res.status(200).json({
    message: `Hello, ${name}!`,
    runtime: "Klyron",
    framework: "Next.js",
    timestamp: new Date().toISOString(),
  });
}
