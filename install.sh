#!/bin/bash

# Create directories if they don't exist
mkdir -p ~/.local/bin ~/.local/share/swaptop

# Copy binary
cp target/release/swaptop ~/.local/bin/

# Make executable
chmod +x ~/.local/bin/swaptop

echo "Swaptop installed to ~/.local/bin/"
echo "Make sure ~/.local/bin/ is in your PATH"