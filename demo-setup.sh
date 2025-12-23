#!/bin/bash
# Setup script to prepare for recording a real Claude CLI session
set -e

echo "Setting up demo environment..."

SRC=${PWD}
# Ensure clean start - remove any existing directory
cd /tmp
rm -rf interminai-demo

# Create fresh demo git repository
mkdir interminai-demo
cd interminai-demo

git init
git config user.name "Demo User"
git config user.email "demo@example.com"

echo "# My Project" > README.md
git add README.md
git commit -m "Initial commit"

echo "print('Hello, World!')" > hello.py
git add hello.py
git commit -m "Add hello world script"

echo "print('Goodbye, World!')" > goodbye.py
git add goodbye.py
git commit -m "Add goodbye script"

echo "print('It's a wonderful world!')" >> hello.py
git add hello.py
git commit -m "Add optimism to hello world script"

echo ""
echo "✓ Demo repository ready at /tmp/interminai-demo"
echo ""
echo "Git log:"
git log --oneline
mkdir -p .claude
cp -fr ${SRC}/skills .claude/
echo "Skills:"
ls .claude/skills
echo ""
echo "Now record your Claude CLI session with:"
echo "  cd /tmp/interminai-demo"
echo "  # Start recording (asciinema or vhs)"
echo "  # Ask Claude: 'Please reorder two commits touching hello world script together: first hello, then goodbye'"
