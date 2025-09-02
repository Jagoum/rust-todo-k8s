/**
 * # Vite Configuration
 * 
 * Build and development server configuration for the React frontend.
 * Configures React plugin, development server settings, and API proxy.
 * 
 * ## Features
 * - React plugin for JSX/TSX support and hot module replacement
 * - Development server with API proxy to backend
 * - Production build optimization
 * - Host binding for Docker container compatibility
 */

import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  // Enable React plugin for JSX/TSX support and fast refresh
  plugins: [react()],
  
  // Development server configuration
  server: {
    // Bind to all interfaces for Docker compatibility
    host: '0.0.0.0',
    // Development server port
    port: 3000,
    // Proxy API requests to backend during development
    proxy: {
      '/api': {
        // Backend server URL
        target: 'http://localhost:8080',
        // Change origin header for CORS
        changeOrigin: true,
        // Remove /api prefix when forwarding to backend
        rewrite: (path) => path.replace(/^\/api/, '')
      }
    }
  },
  
  // Preview server configuration (for production builds)
  preview: {
    // Bind to all interfaces for Docker compatibility
    host: '0.0.0.0',
    // Preview server port
    port: 3000
  }
})