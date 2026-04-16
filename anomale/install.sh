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
Expanding on the philosophy that mangowm offers, anomale shell does **not** include 
a suite of widgets and apps that create a complete desktop environment. 
Instead, it provides a minimal, lightweight, and functional interface 
that provides basic information and wallpaper chooser with 
pywal theming for your minimalistic desktop. 
New features will be added in the future, but the project 
will always maintain that minimalistic philosophy that stays 
out of the user's way and encourages the use of the terminal 
rather than a complicated GUI. Users that are not comfortable working in 
their terminal will likely not enjoy these dots.

While the Anomale Shell source code was included in the 
repo, inside of the shell/ directory, the install script 
is the primary way to install the shell, as it handles the building 
of the binary, installation of any pre-requisites, and the 
copying of configuration files that turn a tedious setup experience 
into a simple 10-minute process.

This Graphical Shell and the included dotfiles 
are meant to be installed over a minimal 
Arch Linux (Arch, CachyOS, EndeavourOS) installation with 
no DE or display manager. (The script may work if used under 
different conditions, but no promises. It requires yay or
paru, or it will install yay for you.)

After considering all of this, you may proceed.
EOF
sleep 0.3

#are you even ready?
cat << "EOF"
Are you Ready to Install Anomale Shell and Jor's Dots?
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
            xargs -a "$THE_STUFF/aurlist.txt" paru -S --needed --skipreview
            sleep 1
            break 
            ;;
        "PLEASE INSTALL YAY FOR ME")
            echo "You don't have either, so the script will install yay for you and use that."
            sudo pacman -S --needed git base-devel
            git clone https://aur.archlinux.org/yay.git
            cd yay
            makepkg -si
            yay -S --needed - < "$THE_STUFF/aurlist.txt"
            cd ..
            rm -rf yay/
            break
            ;;
        *) 
            echo "Invalid entry. Please pick 1 or 2."
            ;;
    esac
done

#set fish as system-wide shell and set local/bin path.
chsh -s /usr/bin/fish
fish -c "fish_add_path ~/.local/bin/"

#Required to build anomale
rustup default stable

#build anomale from source, and copy the binary to ~/.local/bin/ & make executable.
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

#set terminal
fish -c "set -Ux TERMINAL foot"


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

It will be installed with a theme that will 
continuously change with your previosly set wallpaper.
EOF

PS3="Choose: "
options=("YES" "NO")

select opt in "${options[@]}"
do
    case $opt in
        "YES")
            echo "Installing SDDM (you'll have to configure/theme it yourself)"
            sudo pacman -S sddm qt5-graphicaleffects qt5-quickcontrols2 qt5-svg
            sudo systemctl enable sddm
            sudo systemctl set-default graphical.target
            sudo cp -r "$THE_STUFF/anomalous" /usr/share/sddm/themes/
            sudo cp "$THE_STUFF/etc/sddm.conf" /etc/sddm.conf
            sudo chown -R $USER:$USER /usr/share/sddm/themes/anomalous
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
This is the end of the script. Please reboot your computer.
EOF
