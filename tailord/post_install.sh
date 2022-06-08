#!/bin/sh

set -e

# This one of the weirdest hacks I've ever needed.
# Without this, systemd doesn't recognize the unit file
# for some magical reason.
cd $1
cat tailord.service > tailord2.service
mv tailord2.service tailord.service

# This another one of the weirdest hacks I've ever needed.
# Without this, systemd doesn't want to run the executable,
# probably due to SELinux
cd $2
cp tux-tailord tux-tailord2
rm tux-tailord

cp tux-tailord2 tux-tailord
rm tux-tailord2

chmod +x tux-tailord

cd $3
cp com.tux.Tailor.conf tmp.conf
rm com.tux.Tailor.conf 

cp tmp.conf com.tux.Tailor.conf 
rm tmp.conf

systemctl daemon-reload