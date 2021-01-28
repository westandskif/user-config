[[ -f ~/.Xresources ]] && xrdb -merge -I$HOME ~/.Xresources

# xev | grep -A 10 KeyPress

xmodmap -e "keycode 107 = NoSymbol"

dbus-update-activation-environment --systemd DBUS_SESSION_BUS_ADDRESS DISPLAY XAUTHORITY

eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK

# gsettings set org.gnome.mutter experimental-features "['x11-randr-fractional-scaling']"
# gsettings set org.gnome.desktop.interface scaling-factor 1
# gsettings set org.gnome.settings-daemon.plugins.xsettings overrides "{'Xft/DPI': <161613>}"
xsettingsd &

i3 
