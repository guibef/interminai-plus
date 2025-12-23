#!/bin/bash
# Setup script for GDB debugging demo with interminai
set -e

echo "Setting up GDB demo environment..."

SRC=${PWD}
cd /tmp
rm -rf interminai-gdb-demo

mkdir interminai-gdb-demo
cd interminai-gdb-demo

# Create a C program with a function that's called with various parameters
cat > mystery.c << 'EOF'
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// This function processes data - we want to find out what parameters it receives
void process_data(const char *operation, int count, double factor) {
    printf("Processing...\n");
    // Complex logic here...
    for (int i = 0; i < count; i++) {
        // do something
    }
}

void helper(int x) {
    if (x > 10) {
        process_data("multiply", x * 2, 3.14159);
    } else {
        process_data("divide", x, 2.71828);
    }
}

int main(int argc, char *argv[]) {
    int value = 7;
    if (argc > 1) {
        value = atoi(argv[1]);
    }
    
    printf("Starting program with value=%d\n", value);
    helper(value);
    printf("Done.\n");
    return 0;
}
EOF

# Compile with debug symbols
gcc -g -O0 -o mystery mystery.c

echo ""
echo "✓ GDB demo ready at /tmp/interminai-gdb-demo"
echo ""
echo "Program: mystery.c - has a process_data() function called indirectly"
echo "Binary:  mystery (compiled with debug symbols)"
echo ""

# Install skill
mkdir -p .claude
cp -fr ${SRC}/skills .claude/
echo "Skills installed:"
ls .claude/skills
echo ""
echo "Now start Claude and ask:"
echo "  cd /tmp/interminai-gdb-demo"
echo "  claude"
echo ""
echo "  'Use gdb to find out what parameters process_data() is called with when running ./mystery'"
