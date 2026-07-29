/** @type {import('next').NextConfig} */
const nextConfig = {
  // Fail the production build on ESLint errors (Vercel)
  eslint: {
    ignoreDuringBuilds: false,
  },
  // Keep typed routes / strict builds on Vercel
  typescript: {
    ignoreBuildErrors: false,
  },
};

export default nextConfig;
