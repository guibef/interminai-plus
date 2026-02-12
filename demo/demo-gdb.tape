Output demo-gdb.gif

Set FontSize 13
Set Width 800
Set Height 600
Set Theme "VibrantInk"
Set TypingSpeed 100ms

# NOTE: Run ./demo-gdb-setup.sh before running this tape
Type "cd /tmp/interminai-gdb-demo"
Enter
Sleep 1s
Type "pwd"
Enter
Sleep 1s

Type "cat mystery.c"
Enter
Sleep 3s

Type "ls .claude/skills/"
Enter
Sleep 1s

Type "claude --dangerously-skip-permissions"
Enter
Sleep 3s

# User's request to Claude - debug with gdb
Type "Use gdb to find out what parameters process_data() is called with when running ./mystery"
Enter

# Wait for Claude to:
# - Start up and connect
# - Start interminai session with gdb
# - Set breakpoint on process_data
# - Run the program
# - Handle debuginfod prompt if needed
# - Print arguments
# - Report the result
Sleep 180s

# Exit Claude
Ctrl+C
Sleep 2s
