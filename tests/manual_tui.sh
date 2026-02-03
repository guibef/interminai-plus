#!/bin/bash
# tests/manual_tui.sh

# ANSI settings
RESET="\033[0m"
BOLD="\033[1m"
INVERSE="\033[7m"

draw() {
    clear
    echo -e "${BOLD}--- Interminai Parity Test TUI ---${RESET}"
    echo ""
    echo -e "Buttons:      [ Submit ]    ( Cancel )    < Reset >"
    echo ""
    echo -e "Checkboxes:   [ ] Check 1    [x] Check 2    [✓] Check 3"
    echo -e "              ◼ Box 1        ◻ Box 2        ☒ Box 3"
    echo ""
    echo -e "Radios:       ( ) Radio 1    (x) Radio 2    ◉ Selected"
    echo ""
    echo -e "Select Menu:"
    echo -e "  ❯ Entry A"
    echo -e "    Entry B"
    echo -e "    Entry C"
    echo ""
    echo -e "Input Fields: Name: ____________________"
    echo -e "              CMD:  _ "
    echo ""
    echo -e "Progress:     [========>    ] 60%"
    echo -e "Status:       ⠋ Processing...    ${INVERSE} STYLED TAB ${RESET}"
    echo ""
    echo -e "Links:        https://github.com/guibef/interminai-plus"
    echo -e "              src/main.rs:10"
    echo ""
    echo -e "Error:        Error: Invalid operation ✗"
    echo ""
    echo -e "Diff:         + Added line"
    echo -e "              - Removed line"
    echo ""
    echo "Press 'q' to quit. Log: $LOG"
}

LOG="Ready"
while true; do
    draw
    read -rsn1 key
    if [[ $key == "q" ]]; then break; fi

    # Handle escape sequences for arrows
    if [[ $key == $'\e' ]]; then
        read -rsn2 -t 0.1 next
        key+="$next"
    fi

    case "$key" in
        $'\e[A') LOG="UP arrow" ;;
        $'\e[B') LOG="DOWN arrow" ;;
        $'\e[C') LOG="RIGHT arrow" ;;
        $'\e[D') LOG="LEFT arrow" ;;
        $'\r'|'') LOG="ENTER/SELECT" ;;
        ' ')     LOG="SPACE/TOGGLE" ;;
        $'\x01') LOG="CTRL+A (SELECT ALL)" ;;
        $'\x15') LOG="CTRL+U (CLEAR)" ;;
        *)       LOG="Key: $key" ;;
    esac
done
