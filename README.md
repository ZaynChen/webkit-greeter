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
- [x] Greetd support
  - [x] Hyprland. See [greetd.conf](./data/greetd.conf) and [hyprland.conf](./data/hyprland.conf).
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

## Build and install

```sh
git clone https://github.com/ZaynChen/webkit-greeter --recursive
cd webkit-greeter
./install.sh
```

## Theme JavaScript API:

API depends on login manager:
- [greetd.js](./crates/greeters/src/resources/greetd.js) 
- [lightdm.js](./crates/greeters/src/resources/lightdm.js) 

The greeter exposes a JavaScript API to themes which they must use to interact with the greeter (in order to facilitate the user login process). For more details, check out the [LightDM WebKit2 Greeter API Documentation](https://doclets.io/Antergos/lightdm-webkit2-greeter/stable). 

[webkit2-greeter]: https://github.com/Antergos/web-greeter/tree/stable "LightDM WebKit2 Greeter"
[lightdm-webkit-greeter]: https://github.com/ZaynChen/lightdm-webkit-greeter "LightDM WebKit Greeter"
