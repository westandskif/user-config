[[ -f ~/.Xresources ]] && xrdb -merge -I$HOME ~/.Xresources

# xev | grep -A 10 KeyPress
# xmodmap -e "keycode 107 = NoSymbol NoSymbol NoSymbol NoSymbol NoSymbol NoSymbol"

dbus-update-activation-environment --systemd DBUS_SESSION_BUS_ADDRESS DISPLAY XAUTHORITY

eval $(/usr/bin/gnome-keyring-daemon --start --components=pkcs11,secrets,ssh)
export SSH_AUTH_SOCK

xsettingsd &

i3 
