#!/bin/bash
# Demo: Claude using ainter to perform git interactive rebase
set -e

# Save source directory and find ainter binary
SRC=$PWD
AINTER="${SRC}/skills/ainter/scripts/ainter"

if [ ! -x "$AINTER" ]; then
    echo "Error: ainter binary not found at $AINTER"
    echo "Run 'make install-skill' first"
    exit 1
fi

# Setup colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
NC='\033[0m'

echo "# Task: Clean up git history - squash fixup commit"
echo ""
sleep 1

echo "# Setting up demo repository..."
cd /tmp
rm -rf ainter-demo
mkdir ainter-demo
cd ainter-demo
git init
git config user.name "Demo User"
git config user.email "demo@example.com"
echo ""
sleep 0.5

echo "# Creating commits..."
echo "print('Hello, World!')" > hello.py
git add hello.py
git commit -m "Add hello world script"
echo ""

echo "print('Hello, World!')  # Fixed typo" > hello.py
git add hello.py
git commit -m "fixup! Add hello world script"
echo ""
sleep 0.5

echo "# Current git history:"
git log --oneline
echo ""
sleep 1.5

echo "# I need to use 'git rebase -i --autosquash' to squash the fixup commit."
echo "# This opens vim, so I'll use ainter to interact with it."
echo ""
sleep 2

SOCKET=$(mktemp -d /tmp/ainter-XXXXXX)/sock
echo -e "${BLUE}$ GIT_EDITOR=vim ainter start --socket \$SOCKET -- bash${NC}"
GIT_EDITOR=vim "$AINTER" start --socket "$SOCKET" -- bash
sleep 1
echo ""

echo -e "${BLUE}$ printf \"cd /tmp/ainter-demo\\n\" | ainter input --socket \$SOCKET${NC}"
printf "cd /tmp/ainter-demo\n" | "$AINTER" input --socket "$SOCKET"
sleep 0.5

echo -e "${BLUE}$ printf \"git rebase -i --autosquash --root\\n\" | ainter input --socket \$SOCKET${NC}"
printf "git rebase -i --autosquash --root\n" | "$AINTER" input --socket "$SOCKET"
sleep 2
echo ""

echo "# Checking what vim is showing..."
echo -e "${BLUE}$ ainter output --socket \$SOCKET${NC}"
"$AINTER" output --socket "$SOCKET" | head -12
echo ""
sleep 2

echo "# Perfect! The --autosquash flag already set the second commit to 'fixup'."
echo "# I just need to save and exit."
echo ""
sleep 1.5

echo -e "${BLUE}$ printf \":wq\\n\" | ainter input --socket \$SOCKET${NC}"
printf ":wq\n" | "$AINTER" input --socket "$SOCKET"
sleep 1.5
echo ""

echo "# Verifying the result..."
echo -e "${BLUE}$ printf \"git log --oneline\\n\" | ainter input --socket \$SOCKET${NC}"
printf "git log --oneline\n" | "$AINTER" input --socket "$SOCKET"
sleep 0.5
"$AINTER" output --socket "$SOCKET" | grep -A2 "git log"
echo ""
sleep 1.5

echo -e "${GREEN}✓ Success! Two commits squashed into one.${NC}"
echo ""

printf "exit\n" | "$AINTER" input --socket "$SOCKET"
sleep 0.3
"$AINTER" stop --socket "$SOCKET"
rm "$SOCKET"
rmdir "$(dirname "$SOCKET")"
