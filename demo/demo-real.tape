Output demo.gif

Set FontSize 13
Set Width 800
Set Height 600
Set Theme "VibrantInk"
Set TypingSpeed 100ms

# NOTE: Run ./demo-setup.sh before running this tape to ensure clean git repo
Type "cd /tmp/interminai-demo"
Enter
Sleep 1s
Type "pwd"
Enter
Sleep 1s
Type "git log --oneline"
Enter
Sleep 2s

Type "ls .claude/skills/"
Enter
Sleep 1s

Type "claude --dangerously-skip-permissions"
Enter
Sleep 3s

# User's request to Claude - let Claude figure out the approach
Type "Please reorder two commits touching hello world script together: first hello, then goodbye"
Enter

# Wait for Claude to:
# - Start up and connect (25s)
# - Check git log
# - Start interminai session
# - Run git rebase
# - Capture vim output
# - Complete rebase
# - Verify result
Sleep 100s

# Exit Claude
Ctrl+C
Sleep 2s

# Verify the result (commented out - Claude does this itself)
# Type "git log --oneline"
# Enter
# Sleep 15s
