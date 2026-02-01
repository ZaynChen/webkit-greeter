#!/usr/bin/env bash

_pkgname="webkit-greeter"
if [ -d /usr/local/lib/$_pkgname ]; then
  sudo rm  /usr/local/bin/$_pkgname
  sudo rm -r /usr/local/lib/$_pkgname
  sudo rm -r /usr/local/etc/$_pkgname
  sudo rm -r /usr/local/share/$_pkgname
  sudo rm /usr/local/lib/sysusers.d/$_pkgname.conf
  sudo rm /usr/local/lib/tmpfiles.d/$_pkgname.conf
  sudo rm /usr/share/xgreeters/$_pkgname.desktop
fi

sudo rm "/usr/bin/$_pkgname"
sudo rm -r "/usr/lib/$_pkgname"
sudo rm -r "/etc/$_pkgname"
sudo rm -r "/usr/share/doc/$_pkgname"
sudo rm -r "/usr/share/licenses/$_pkgname"
sudo rm "/usr/lib/sysusers.d/$_pkgname.conf"
sudo rm "/usr/lib/tmpfiles.d/$_pkgname.conf"
