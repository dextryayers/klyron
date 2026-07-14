/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: "standalone",
  experimental: {
    // Klyron-compatible settings
    serverComponentsExternalPackages: [],
  },
};

export default nextConfig;
