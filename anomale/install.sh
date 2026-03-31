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
ever so slightly on that philosophy. 

This script should run smoothly on Arch, CachyOS, and EndeavourOS.
The script is meant to be installed on top of Desktopless or Minimal OS installs, but may work regardless.
However, it assumes you handle graphics drivers YOURSELF.

This script installs the latest build of mangowm for you from the AUR, and does not have 
any other pre-requisites. If you have neither yay or paru installed prior to running this script, yay will be installed for you.

These dots and this configuration are based on my personal preference and NOTHING more. My default mango bindings
may not be to your liking. Anomale-specific bindings will be in a single section of the config file with brief explanations.

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
echo "This script requires sudo for various commands during setup. You may be asked several times throughout."
sleep 3
sudo -v 

clear
cat << "EOF"
These Dots should work well on Arch Linux, EndeavourOS, and CachyOS.
Howevever, this script must do a few things differently depending on your OS's default configuration. 

(If you have EndeavourOS, unless you unselected it during install, you already have Yay.)
(If you have CachyOS, you have paru installed already.)
(If you have base Arch Linux, You don't have either by default)

EOF

PS3="Choose: "
options=("I HAVE YAY" "I HAVE PARU" "PLEASE INSTALL YAY FOR ME")

select opt in "${options[@]}"
do
    case $opt in
        "I HAVE YAY")
            echo "You have yay, so the script will use that."
            yay -S --needed - < "$THE_STUFF/aurlist.txt"
            sleep 1
            break 
            ;;
        "I HAVE PARU")
            echo "You have paru, so the script will use that."
            paru -S --needed --skipreview - < "$THE_STUFF/aurlist.txt"
            sleep 1
            break 
            ;;
        "PLEASE INSTALL YAY FOR ME")
            echo "You don't have either, so the script will install yay for you and use that."
            git clone https://aur.archlinux.org/yay.git
            cd yay
            makepkg -si
            cd ~
            rm -rf ~/yay
            yay -S --needed - < "$THE_STUFF/aurlist.txt"
            break
            ;;
        *) 
            echo "Invalid entry. Please pick 1 or 2."
            ;;
    esac
done

#set fish as system-wide shell and set local/bin path.
chsh -s /usr/bin/fish
fish_add_path ~/.local/bin

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


PS3="DO YOU HAVE NVIDIA GPU?: "
options=("YES" "NO")

select opt in "${options[@]}"
do
    case $opt in
        "YES")
            echo "sorry..."
            rm -f ~/.local/bin/mangowc-start-nonvidia.sh
            mv ~/.local/bin/mangowc-start-nvidia.sh ~/.local/bin/mangowc-start.sh
            sleep 1
            break 
            ;;
        "NO")
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
chmod +x ~/.local/bin/*
clear

cat << "EOF"
Would you like SDDM?
EOF

PS3="Choose: "
options=("YES" "NO")

select opt in "${options[@]}"
do
    case $opt in
        "YES")
            echo "Installing SDDM (you'll have to configure/theme it yourself)"
            sudo pacman -S sddm
            sudo systemctl enable sddm
            sudo systemctl set-default graphical.target
            break 
            ;;
        "NO")
            echo "You'll have to log in from TTY if you do not set up a display manager on your own."
            break
            ;;
        *) 
            echo "Invalid entry. Please pick 1 or 2."
            ;;
    esac
done
clear

cat << "EOF"
The Script has completed successfully and you are very, very happy about it.

Do yourself a favor and reboot your computer. 

When you get back, you'll find that basic sddm theme. Its okay. You hate it, I hate it.
We can hate it together.
EOF
