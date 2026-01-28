#!/usr/bin/env bash

build() {
  cargo build --release --locked
}

package() {
  sudo install -Dm0755 target/release/libwebkit_greeter_webext.so /usr/local/lib/webkit-greeter/libwebkit-greeter-webext.so
  sudo install -Dm0755 target/release/webkit-greeter /usr/local/bin/webkit-greeter

  sudo install -Dm0644 data/webkit-greeter.desktop /usr/local/share/greeters/webkit-greeter.desktop
  # lightdm need
  sudo install -Dm0644 data/webkit-greeter.desktop /usr/share/xgreeters/webkit-greeter.desktop

  sudo install -Dm0644 data/greetd.conf /etc/greetd/greetd.conf
  sudo install -Dm0644 data/hyprland.conf /usr/local/etc/greetd/hyprland.conf
  sudo install -Dm0644 data/webkit-greeter.toml /usr/local/etc/greetd/webkit-greeter.toml

  sudo install -Dm0644 data/greetd.pam /etc/pam.d/greetd
  sudo install -Dm0644 data/webkit-greeter.sysusers /usr/local/lib/sysusers.d/webkit-greeter.conf
  sudo install -Dm0644 data/webkit-greeter.tmpfiles /usr/local/lib/tmpfiles.d/webkit-greeter.conf

  # create sysusers
  sudo systemd-sysusers /usr/local/lib/sysusers.d/webkit-greeter.conf
}

build

package
