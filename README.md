# ANOMALE SHELL
<a href="https://www.youtube.com/watch?v=IXHZVE5SDYE">
  <img src="https://img.youtube.com/vi/IXHZVE5SDYE/maxresdefault.jpg" width="700" alt="Watch a preview video.">
</a>

THIS PROJECT IS A PERSONAL PROJECT WITH SOFTWARE AND DOTFILES MADE FOR MYSELF. IT IS NOT INTENDED FOR PUBLIC USE, ALTHOUGH YOU ARE WELCOME TO USE IT IF YOU WISH.

Expanding on the philosophy that mangowm offers, anomale shell does **not** include a suite of widgets and apps that create a complete desktop environment. Instead, it provides a minimal, lightweight, and functional interface that provides basic information and wallpaper chooser with pywal theming for your minimalistic desktop. New features will be added in the future, but the project will always maintain that minimalistic philosophy that stays out of the user's way and encourages the use of the terminal rather than a complicated GUI. Users that are not comfortable working in their terminal will likely not enjoy these dots.

While the Anomale Shell source code is included in this repo inside of the shell/ directory, the install script is the primary way to install the shell, as it handles the building of the binary, installation of any pre-requisites, and the copying of configuration files that turn a tedious setup experience into a simple 10-minute process.

This Graphical Shell and the included dotfiles are meant to be installed over a minimal Arch Linux (Arch, CachyOS, EndeavourOS) installation with no DE or display manager. (The script may work if used under different conditions, but no promises. It requires yay or paru, or it will install yay for you.)


## Installation

To install the shell, run the following commands:

**STEP 1:**

```bash
sudo pacman -S --needed git base-devel
```

**STEP 2:**

```bash
git clone https://github.com/thatsjor/anomale-shell.git
```

**STEP 3:**

```bash
chmod +x anomale-shell/anomale/install.sh
```

**STEP 4:**

```bash
bash anomale-shell/anomale/install.sh
```

The setup will require sudo permissions. Pay attention throughout the process as you'll be needed for various prompts and confirmations. You will be given the options at the end to add Nvidia environment variables to your autostart script and install/configure SDDM before you reboot.
