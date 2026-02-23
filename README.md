<div align="center">
  <h1><strong>WebKit Greeter</strong></h1>
  <p>
    <strong>greeter made with WebKitGTK</strong>
  </p>
</div>

WebKit Greeter allows to create themes with web technologies, similar as the outdated
[lightdm-webkit2-greeter][webkit2-greeter]. This project is based on [lightdm-webkit-greeter][lightdm-webkit-greeter].

## Status

webkit-greeter is **alpha** project. It is unfinished and subject to constant breaking changes.
So it is not recommended to use it as your main greeter yet. However, it is functional,
I use it as my main greeter for greetd with hyprland as compositor.

- [x] Multi-monitor support.
- [x] Detect theme errors prompt
- [x] Add themes
- [x] Add config
- [x] Greetd support. See [greetd.conf](./examples/greetd.conf)
  - [x] Hyprland. See [hyprland.conf](./examples/hyprland.conf).
  - [x] Sway. See [sway.conf](./examples/sway.conf).
- [x] Lightdm support
- [ ] Brightness feature support
- [ ] Battery feature support

## Dependencies

- gtk4
- webkitgtk-6.0
- systemd
- accountsservice (optional)
- greetd (optional)
- lightdm (optional)

### Build dependencies

- Rust
- Cargo
- npm

## Installation
### From source

```sh
git clone https://github.com/ZaynChen/webkit-greeter --recursive
cd webkit-greeter
./install.sh
```

Manifest:
- `/usr/bin/webkit-greeter`
- `/usr/lib/webkit-greeter/libwebext.so`
- `/etc/webkit-greeter/webkit-greeter.toml`
- `/usr/lib/sysusers.d/webkit-greeter.conf`
- `/usr/lib/tmpfiles.d/webkit-greeter.conf`
- `/usr/share/doc/webkit-greeter/examples/greetd.conf`
- `/usr/share/doc/webkit-greeter/examples/greetd.pam`
- `/usr/share/doc/webkit-greeter/examples/hyprland.conf`
- `/usr/share/doc/webkit-greeter/examples/sway.conf`
- `/usr/share/doc/webkit-greeter/examples/webkit-greeter.desktop`
- `/usr/share/webkit-greeter/themes/litarvan/*`

Details:
- [webkit-greeter.sysusers](./data/webkit-greeter.sysusers) is used for `systemd-sysusers` to create `'webkit-greeter'` user for greeter session.
- [webkit-greeter.tmpfiles](./data/webkit-greeter.tmpfiles) is used for `systemd-tmpfiles` to create necessary directories for `'webkit-greeter'` user to access.
  - `/var/lib/webkit-greeter` is the home directory of user `'webkit-greeter'`, in which store the cache and state data of webkit-greeter.
  - `/var/log/webkit-greeter` is the log directory for webkit-greeter to store the output log file. (webkit-greeter currently output log message to `stderr`, so you may need to store log file manually, see [examples/hyprland.conf](./examples/hyprland.conf))

## Usage

You can get example config file under [examples](./examples) directory:
- [examples/greetd.conf](./examples/greetd.conf)
- [examples/hyprland.conf](./examples/hyprland.conf)
- [examples/sway.conf](./examples/sway.conf)

After installation, these examples will be placed in `/usr/share/doc/webkit-greeter/examples/` directory.

### Greetd

Edit the greetd config file `/etc/greetd/greetd.conf` to set webkit-greeter with a Wayland compositor as the default session.

[greetd.conf](./examples/greetd.conf) example for Hyprland:
```ini
[default_session]
command = "Hyprland --config /etc/webkit-greeter/hyprland.conf"
user = "webkit-greeter"
```

Create a Hyprland config file (in a path such as `/etc/webkit-greeter/hyprland.conf`) as follows (full example in [hyprland.conf](./examples/hyprland.conf)):

```sh
$log=/var/log/webkit-greeter/greeter.log
$old=/var/log/webkit-greeter/greeter.old.log
exec-once = mv $log $old; webkit-greeter 2>$log; hyprctl dispatch exit
misc {
  disable_hyprland_logo = true
  disable_splash_rendering = true
  # Set focus_on_activate to true, 
  # so that Hyprland will focus an app that requests to be focused.
  # With this webkit-greeter can grab the focus
  focus_on_activate = true
}
```

Similar example for Sway (full example in [sway.conf](./examples/sway.conf)):

```sh
exec "mv /var/log/webkit-greeter/greeter.log /var/log/webkit-greeter/greeter.old.log;\
webkit-greeter 2>/var/log/webkit-greeter/greeter.log; swaymsg exit"
include /etc/sway/config.d/*
```

Replace `/etc/pam.d/greetd` with [greetd.pam](./examples/greetd.pam), which [Unlocking keyring](https://wiki.archlinux.org/title/Greetd#Unlocking_keyring_on_autologin_using_the_cryptsetup_password), so that after login, you don't need to input the passwd to a polkit prompt for unloking keyring when needed.

### LightDM

Copy [webkit-greeter.desktop](./examples/webkit-greeter.desktop) to `/usr/share/xgreeters` for lightdm detecting the greeter.

Edit `/etc/lightdm/lightdm.conf` to use webkit-greeter with the following:

```ini
[Seat:*]
greeter-session=webkit-greeter
```

## Theme JavaScript API:

API depends on login manager:
- [greetd.js](./themes/webkit-greeter-api/greetd.js) 
- [lightdm.js](./themes/webkit-greeter-api/lightdm.js) 

The greeter exposes a JavaScript API to themes which they must use to interact with the greeter (in order to facilitate the user login process). For more details, check out the [LightDM WebKit2 Greeter API Documentation](https://doclets.io/Antergos/lightdm-webkit2-greeter/stable). 

[webkit2-greeter]: https://github.com/Antergos/web-greeter/tree/stable "LightDM WebKit2 Greeter"
[lightdm-webkit-greeter]: https://github.com/ZaynChen/lightdm-webkit-greeter "LightDM WebKit Greeter"
