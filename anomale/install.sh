#!/bin/bash

#clear tty and define variables
clear
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
THE_STUFF="$SCRIPT_DIR/thestuff" 

#welcome to the installer, kid... Have a disclaimer.
echo -e "\033[0;32m" 
cat << "EOF"

Oh wow, I guess you're trying to install...
 _____                                                  _____ 
( ___ )                                                ( ___ )
 |   |~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~|   | 
 |   |     _    _   _  ___  __  __    _    _     _____  |   | 
 |   |    / \  | \ | |/ _ \|  \/  |  / \  | |   | ____| |   | 
 |   |   / _ \ |  \| | | | | |\/| | / _ \ | |   |  _|   |   | 
 |   |  / ___ \| |\  | |_| | |  | |/ ___ \| |___| |___  |   | 
 |   | /_/__ \_\_|_\_|\___/|_|  |_/_/   \_\_____|_____| |   | 
 |   | / ___|| | | | ____| |   | |                      |   | 
 |   | \___ \| |_| |  _| | |   | |                      |   | 
 |   |  ___) |  _  | |___| |___| |___                   |   | 
 |   | |____/|_| |_|_____|_____|_____| With Jor's Dots! |   | 
 |___|~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~|___| 
(_____)                                                (_____)

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
ever so slightly on that philosophy. This script is intended to be run on a FRESH installation of EndeavourOS,
installed with 'No Desktop'. If you are NOT on a fresh installation of EndeavourOS or do not 
already have yay installed, exit this script and install yay, as it is required for this script to work.
This script installs the latest build of mangowm for you from the AUR, and does not have 
any other pre-requisites.

These dots and this configuration are based on my personal preference and NOTHING more. My default mango bindings
may not be to your liking. Anomale-specific bindings will be in a single section of the config file with brief explanations.

Untested on other Arch-Based distros.

If you understand this, and you're feeling brave, you may proceed...
EOF
sleep 0.3

#are you even ready?
cat << "EOF"
FEELING BRAVE?
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

#initialization - perms request
echo "Starting the installation..."
echo "This script requires sudo for various commands during setup. 
This first Section will install all necessary packages and run some setup commands.
You may be asked for your password multiple times.
Pay attention during the setup process, while you can spam Enter for most of it (works on my machine),
only you know what configuration you need."
sudo -v 

#installs packages necessary for my dots and anomale
yay -S --needed - < "$THE_STUFF/aurlist.txt"

#installs fisher and tide without triggering any config wizards
fish -c "set -U tide_install_no_configure; fisher install jorgebucaran/fisher IlanCosman/tide@v6"

#set fish as system-wide shell
chsh -s /usr/bin/fish

#Required to build anomale
rustup default stable

#create anomale directory
mkdir "$THE_STUFF/shell/"

#extract anomale to thestuff/shell/, build it, and copy the binary to ~/.local/bin/ & make executable.
tar -xvzf "$THE_STUFF/anomale-source.tar.gz" -C "$THE_STUFF/shell/"
(cd "$THE_STUFF/shell/" && cargo build --release)
mkdir -p ~/.local/bin/
cp "$THE_STUFF/shell/target/release/anomale" ~/.local/bin/
chmod +x ~/.local/bin/anomale

#copy wallpaper/ .config .local & .cache contents
mkdir -p ~/Pictures/wallpaper/
cp -r "$THE_STUFF/wallpaper/." ~/Pictures/wallpaper/

mkdir -p ~/.cache/
cp -r "$THE_STUFF/.cache/." ~/.cache/

mkdir -p ~/.config/
cp -r "$THE_STUFF/.config/." ~/.config/

mkdir -p ~/.local/bin/
cp -r "$THE_STUFF/.local/bin/." ~/.local/bin/

chmod +x ~/.local/bin/*

#gtk pywal symlinks
rm -f ~/.config/gtk-4.0/gtk.css
rm -f ~/.config/gtk-4.0/gtk-dark.css
rm -f ~/.config/gtk-3.0/gtk.css
rm -f ~/.config/gtk-3.0/gtk-dark.css

ln -s ~/.cache/wal/gtk-css.css ~/.config/gtk-4.0/gtk.css
ln -s ~/.cache/wal/gtk-css.css ~/.config/gtk-4.0/gtk-dark.css
ln -s ~/.cache/wal/gtk-css.css ~/.config/gtk-3.0/gtk.css
ln -s ~/.cache/wal/gtk-css.css ~/.config/gtk-3.0/gtk-dark.css

#install font and some python packages
getnf -i 0xProto
pip install colorz --break-system-packages

#nvidia check
clear
cat << "EOF"
To make sure your environment variables in your autostart script are configured properly, Please Share whether 
or not you suffer from "I have an NVidia GPU and Use Linux" disorder.
EOF

PS3="Choose: "
options=("I Have A NVidia GPU" "I Do Not have A NVidia GPU")

select opt in "${options[@]}"
do
    case $opt in
        "I Have A NVidia GPU")
            echo "sorry..."
            rm -f ~/.local/bin/mangowc-start-nonvidia.sh
            mv ~/.local/bin/mangowc-start-nvidia.sh ~/.local/bin/mangowc-start.sh
            sleep 1
            break 
            ;;
        "I Do Not have A NVidia GPU")
            echo "lucky..."
            rm -f ~/.local/bin/mangowc-start-nvidia.sh
            mv ~/.local/bin/mangowc-start-nonvidia.sh ~/.local/bin/mangowc-start.sh
            sleep 1
            break
            ;;
        *) 
            echo "Invalid entry. Please pick 1 or 2."
            ;;
    esac
done
clear
chmod +x ~/.local/bin/*

#sddm must be installed for some things to work. no theme yet, just testing.
sudo pacman -S sddm
sudo systemctl enable sddm
sudo systemctl set-default graphical.target

clear

cat << "EOF"
The Script has completed successfully and you are very, very happy about it.

Do yourself a favor and reboot your computer. 

When you get back, you'll find that basic sddm theme. Its okay. You hate it, I hate it.
We can hate it together.
EOF
