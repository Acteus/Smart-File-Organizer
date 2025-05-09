#!/bin/bash

# Install Rust dependencies
echo "Installing Rust dependencies..."
cd src-tauri
cargo update
cd ..

# Install Node dependencies
echo "Installing Node dependencies..."
npm install

# Install TailwindCSS and related tools
echo "Setting up TailwindCSS..."
npm install -D tailwindcss autoprefixer postcss
npm install lucide-svelte svelte-dnd-action

# Make the script executable
chmod +x install.sh

# Complete message
echo "Installation complete!"
echo "To start the development server, run: npm run tauri dev" 