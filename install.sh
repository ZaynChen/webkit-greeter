#!/usr/bin/env bash

if ( ! command -v systemctl >/dev/null 2>&1 ) ; then
  echo "systemd not installed"
  exit 1
fi

if ( ! command -v systemctl status accounts-daemon >/dev/null 2>&1 ) ; then
  echo "accountsservice not installed"
  exit 1
fi

echo "1) auto-detect  2) greetd  3) lightdm  4) all"
read -p "Select the display manager [1]: " opt
case ${opt:-1} in
  1)
    dm=$(systemctl --property=Id show display-manager | cut -d '=' -f 2 | cut -d '.' -f 1) ;;
  2)
    dm="greetd" ;;
  3)
    dm="lightdm" ;;
  4)
    dm="all" ;;
  *)
    echo "Sorry, wrong selection $REPLY"
    exit 1 ;;
esac
echo "Display manager: $dm"

_pkgname="webkit-greeter"

build() {
  cargo build --release --locked --no-default-features --features $_pkgname/$dm

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  ./build.sh
  cd $CURR_DIR
}

package() {
  sudo install -Dm0755 "target/release/$_pkgname" "/usr/bin/$_pkgname"
  sudo install -Dm0755 "target/release/libwebext.so" "/usr/lib/$_pkgname/libwebext.so"

  sudo install -Dm0644 "data/$_pkgname.toml" "/etc/$_pkgname/$_pkgname.toml"
  sudo install -Dm0644 "data/$_pkgname.sysusers" "/usr/lib/sysusers.d/$_pkgname.conf"
  sudo install -Dm0644 "data/$_pkgname.tmpfiles" "/usr/lib/tmpfiles.d/$_pkgname.conf"
  # create sysusers
  sudo systemd-sysusers "/usr/lib/sysusers.d/$_pkgname.conf"

  sudo install -Dm0644 examples/* -t "/usr/share/doc/$_pkgname/examples"
  sudo install -Dm0644 LICENSE -t "/usr/share/licenses/$_pkgname"

  if [ $dm = greetd ] || [ $dm = all ] ; then
    _exampledir="/usr/share/doc/$_pkgname/examples"
    sudo cp "$_exampledir/greetd.conf" "/etc/greetd/greetd.conf"
    sudo cp "$_exampledir/greetd.pam" "/etc/pam.d/greetd"
    sudo cp "$_exampledir/hyprland.conf" "/etc/webkit-greeter/hyprland.conf"
    sudo cp "$_exampledir/sway.conf" "/etc/webkit-greeter/sway.conf"
  fi

  if [ $dm = lightdm ] || [ $dm = all ] ; then
    sudo cp "$_pkgdocdir/$_pkgname.desktop" "/usr/share/xgreeters/$_pkgname.desktop"
  fi

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  _themepkg="lightdm-webkit-theme-litarvan-$(cat version).tar.gz"
  sudo install -Dm0755 $_themepkg -t "/usr/share/$_pkgname/themes/litarvan/"
  sudo install -Dm0644 LICENSE -t "/usr/share/licenses/lightdm-webkit-theme-litarvan/"
  cd /usr/share/$_pkgname/themes/litarvan/
  sudo tar -xvf $_themepkg
  cd $CURR_DIR
}

build

package
