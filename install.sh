#!/usr/bin/env bash

if ( ! command -v systemctl >/dev/null 2>&1 ) ; then
  echo "systemd not installed"
  exit 1
fi

if ( ! command -v WebKitWebDriver >/dev/null 2>&1 ) ; then
  echo "webkitgtk not installed"
  exit 1
fi

_pkgname="webkit-greeter"

build() {
  cargo build --release --locked

  CURR_DIR=$(pwd)
  cd themes/litarvan
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

  CURR_DIR=$(pwd)
  cd themes/litarvan
  _themepkg="*-$(cat version).tar.gz"
  sudo install -Dm0755 $_themepkg -t "/usr/share/$_pkgname/themes/litarvan/"
  cd /usr/share/$_pkgname/themes/litarvan/
  sudo tar -xvf $_themepkg
  cd $CURR_DIR
}

build

package
