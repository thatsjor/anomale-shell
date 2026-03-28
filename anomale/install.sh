#!/bin/bash
clear
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
THE_STUFF="$SCRIPT_DIR/thestuff" 

echo -e "\033[0;32m" 
cat << "EOF"

Oh wow, I guess 
you're trying
to install...
┏┓┳┓┏┓┳┳┓┏┓┓ ┏┓ 
┣┫┃┃┃┃┃┃┃┣┫┃ ┣  
┛┗┛┗┗┛┛ ┗┛┗┗┛┗┛ 
      ┏┓┓┏┏┓┓ ┓ 
      ┗┓┣┫┣ ┃ ┃ 
      ┗┛┛┗┗┛┗┛┗┛

Nice...
EOF
sleep 1
echo -e "\033[0m" 

sleep 0.5

cat << "EOF"
Before we get started, It is important that you understand that this script 
and the software and configuration it installs on your system are in very early stages 
of development. Treat it as incomplete software, and expect bugs.

This shell is built to provide an EXTREMELY minimal experience for users
that are comfortable working in their terminal. These are not your average dotfiles with 
various widgets and menus that make a somewhat funcitonal Desktop Environment.

This shell and these dots appreciate the simplicity that mangowm provides its users, and expands 
ever so slightly on that philosophy. This installer is intended to be run on a FRESH installation of EndeavourOS 
installed with No Desktop. If you are NOT on a fresh installation of EndeavourOS or do not 
already have yay installed, exit this script and install yay, as it is required for this script to work.
This script installs the latest build of mangowm for you from the AUR, and does not have 
any other pre-requisites.

These dots and this configuration are based on my personal preference and NOTHING more. My default mango bindings
may not be to your liking. Anomale-specific bindings will be in a single section of the config file with brief explanations.

Untested on other Arch-Based distros.

If you understand this, and you're feeling brave, you may proceed...

EOF
sleep 0.3
cat << "EOF"
╭─╴╭─╴╭─╴╷  ╷╭╮╷╭─╴   ╭╮ ╭─╮╭─╮╷ ╷╭─╴╭─╮
├╴ ├╴ ├╴ │  ││╰┤│╶╮   ├┴╮├┬╯├─┤│╭╯├╴  ╶╯
╵  ╰─╴╰─╴╰─╴╵╵ ╵╰─╯   ╰─╯╵╰╴╵ ╵╰╯ ╰─╴ ╵ 
EOF

PS3="Choose (but don't be a coward): "
options=("LETS DO THIS" "GET ME OUTTA HERE")

select opt in "${options[@]}"
do
    case $opt in
        "LETS DO THIS")
            echo "Nice..."
            sleep 1
            break 
            ;;
        "GET ME OUTTA HERE")
            echo "Safe choice. No changes were made. Exiting..."
            sleep 1
            exit 0 
            ;;
        *) 
            echo "Invalid entry. Please pick 1 or 2."
            ;;
    esac
done

clear
echo "Starting the installation..."
