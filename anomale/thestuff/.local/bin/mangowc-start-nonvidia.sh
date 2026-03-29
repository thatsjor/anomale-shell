 #!/bin/bash


export XDG_SESSION_TYPE=wayland

export ELECTRON_OZONE_PLATFORM_HINT=auto

export NVD_BACKEND=direct

export XDG_CURRENT_DESKTOP=wlroots


eval $(gnome-keyring-daemon --start --components=secrets)

export SQLITE_TMPDIR=/tmp


/usr/lib/polkit-gnome/polkit-gnome-authentication-agent-1 &


anomale &


dbus-update-activation-environment --systemd --all  
