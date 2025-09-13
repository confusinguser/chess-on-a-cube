#!/bin/bash

# Build the WASM package
echo "Building WASM package..."
wasm-pack build --target web --out-dir pkg

# Copy assets to the pkg directory for web deployment
echo "Copying assets..."
if [ -d "assets" ]; then
    cp -r assets pkg/
fi

echo "Build complete! You can now serve the files with a web server."
echo "For example, run: python3 -m http.server 8000"
echo "Then open http://localhost:8000 in your browser"
