#!/usr/bin/env bash

if ( ! command -v systemctl >/dev/null 2>&1 ) ; then
  echo "systemd not installed"
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

build() {
  cargo build --release --locked --no-default-features --features webkit-greeter/$dm

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  ./build.sh
  cd $CURR_DIR
}

package() {
  if [ $dm = greetd ] || [ $dm = all ] ; then
    if ( ! command -v systemctl status accounts-daemon >/dev/null 2>&1 ) ; then
      echo "accountsservice not installed"
      exit 1
    fi

    sudo install -Dm0644 data/greetd.conf /etc/greetd/greetd.conf
    sudo install -Dm0644 data/webkit-greeter.toml /usr/local/etc/greetd/webkit-greeter.toml

    sudo install -Dm0644 data/hyprland.conf /usr/local/etc/greetd/hyprland.conf
    sudo install -Dm0644 data/sway.conf /usr/local/etc/greetd/sway.conf

    sudo install -Dm0644 data/greetd.pam /etc/pam.d/greetd
    sudo install -Dm0644 data/webkit-greeter.sysusers /usr/local/lib/sysusers.d/webkit-greeter.conf
    sudo install -Dm0644 data/webkit-greeter.tmpfiles /usr/local/lib/tmpfiles.d/webkit-greeter.conf

    # create sysusers
    sudo systemd-sysusers /usr/local/lib/sysusers.d/webkit-greeter.conf
  fi

  if [ $dm = lightdm ] || [ $dm = all ] ; then
    sudo install -Dm0644 data/webkit-greeter.desktop /usr/share/xgreeters/webkit-greeter.desktop
    sudo install -Dm0644 data/webkit-greeter.toml /etc/lightdm/webkit-greeter.toml
  fi

  sudo install -Dm0755 target/release/libwebkit_greeter_webext.so /usr/local/lib/webkit-greeter/libwebkit-greeter-webext.so
  sudo install -Dm0755 target/release/webkit-greeter /usr/local/bin/webkit-greeter

  [ -d /usr/local/share/webkit-greeter/themes/ ] || sudo mkdir -p /usr/local/share/webkit-greeter/themes/

  CURR_DIR=$(pwd)
  cd themes/lightdm-webkit-theme-litarvan
  VERSION=$(cat version)
  sudo rm -r /usr/local/share/webkit-greeter/themes/litarvan/
  sudo mkdir /usr/local/share/webkit-greeter/themes/litarvan/
  sudo cp ./lightdm-webkit-theme-litarvan-$VERSION.tar.gz /usr/local/share/webkit-greeter/themes/litarvan/

  cd /usr/local/share/webkit-greeter/themes/litarvan/
  sudo tar -xvf lightdm-webkit-theme-litarvan-$VERSION.tar.gz
  cd $CURR_DIR
}

build

package
