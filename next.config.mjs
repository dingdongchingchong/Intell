/** @type {import('next').NextConfig} */
const nextConfig = {
  eslint: {
    ignoreDuringBuilds: false,
  },
  typescript: {
    ignoreBuildErrors: false,
  },
  async rewrites() {
    return [
      // Prefer the original Investigation Manager SPA at the site root path alias
      { source: "/manager", destination: "/investigation.html" },
      { source: "/app", destination: "/investigation.html" },
    ];
  },
};

export default nextConfig;
