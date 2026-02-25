import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  async rewrites() {
    return [
      {
        // When the frontend fetches /api/path...
        source: "/api/:path*",
        // ...proxy the request to the Rust backend
        destination: "http://127.0.0.1:3000/:path*",
      },
    ];
  },
};

export default nextConfig;
