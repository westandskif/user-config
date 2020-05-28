[[ -f ~/.Xresources ]] && xrdb -merge -I$HOME ~/.Xresources

xmodmap -e "keycode 94 = NoSymbol"

dbus-update-activation-environment --systemd DBUS_SESSION_BUS_ADDRESS DISPLAY XAUTHORITY

/usr/bin/dunst &

eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK
i3
