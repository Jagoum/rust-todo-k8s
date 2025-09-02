/**
 * # Application Entry Point
 * 
 * Main entry point for the React application. Sets up the root component
 * with React StrictMode for development checks and warnings.
 * 
 * ## Setup
 * - Mounts the App component to the DOM element with id 'root'
 * - Enables React StrictMode for additional development checks
 * - Imports global CSS styles
 */

import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './index.css'

// Create React root and render the application
ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)