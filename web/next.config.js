/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "export",
  basePath: "/pymermaider",
  images: {
    unoptimized: true,
  },
  webpack: (config) => {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };
    return config;
  },
  // Disable trailing slashes
  trailingSlash: false,
};

module.exports = nextConfig;
