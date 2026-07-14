export async function benchCache(): Promise<void> {
  const start = Date.now();
  // Benchmark logic
  const elapsed = Date.now() - start;
  console.log(`${elapsed}ms`);
}
