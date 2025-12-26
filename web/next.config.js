/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
  // Prefer explicit base path from the environment (used by GitHub Pages),
  // but fall back to the historical production default.
  basePath:
    process.env.NEXT_PUBLIC_BASE_PATH ??
    (process.env.NODE_ENV === "production" ? "/pymermaider" : ""),
  turbopack: {
    // This repo has multiple lockfiles (workspace + web/). Pin the root so Next
    // doesn't warn/mis-detect the workspace root.
    root: __dirname,
  },
  images: {
    unoptimized: true,
  },
  // Disable trailing slashes
  trailingSlash: false,
};

module.exports = nextConfig;
