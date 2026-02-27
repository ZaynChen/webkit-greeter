// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

import { assert, assertEquals, assertFalse } from "@std/assert";
import "./mod.ts";
import "../mock.ts";

assertEquals(greeter_config.greeter, {
  debug_mode: true,
  detect_theme_errors: true,
  screensaver_timeout: 300,
  secure_mode: true,
  theme: "litarvan",
  icon_theme: null,
  time_language: null,
});
assertEquals(greeter_config.branding, {
  background_images_dir: "/user/share/backgrounds",
  logo_image: "./assets/logo.png",
  user_image: "./assets/default_user.png",
});
assertEquals(greeter_config.layouts, [{
  name: "us",
  description: "English (US)",
  short_description: "en",
}]);
assertEquals(await theme_utils.dirlist("/usr/share/backgrounds", true), [
  "/usr/share/backgrounds/archlinux/archbtw.png",
  "/usr/share/backgrounds/archlinux/awesome.png",
]);
assertEquals(await theme_utils.dirlist("/usr/share/backgrounds", false), [
  "/usr/share/backgrounds/archlinux/archlinux.stw",
  "/usr/share/backgrounds/archlinux/archbtw.png",
  "/usr/share/backgrounds/archlinux/awesome.png",
]);
assertEquals(
  greeter_comm.broadcast({
    "type": "change-background",
    "path": "/usr/share/backgrounds/archlinux/awesome.png",
  }),
  null,
);

assertEquals(greeter.language, {
  code: "en_US.utf8",
  name: "American English",
  territory: "United States",
});
assertEquals(greeter.languages, [
  {
    name: "American English",
    code: "en_US.utf8",
    territory: "United States",
  },
  {
    name: "Français",
    code: "fr_FR.utf8",
    territory: "",
  },
  {
    name: "中文",
    code: "zh_CN.utf8",
    territory: "中国",
  },
]);
assertEquals(greeter.layout, {
  name: "us",
  description: "English (US)",
  short_description: "en",
});
assertEquals(greeter.layouts, [
  {
    name: "us",
    description: "English (US)",
    short_description: "en",
  },
]);
assertEquals(greeter.sessions, [
  { name: "KDE 5", key: "plasma-shell", type: "", comment: "" },
  { name: "Gnome 3", key: "gnome-shell", type: "", comment: "" },
  { name: "XFCE 4", key: "xfce", type: "", comment: "" },
  { name: "Cinnamon", key: "cinnamon", type: "", comment: "" },
  { name: "i3", key: "i3", type: "", comment: "" },
  { name: "Hyprland", key: "hyprland", type: "", comment: "" },
  { name: "xmonad", key: "xmonad", type: "", comment: "" },
  { name: "Qtile", key: "qtile", type: "", comment: "" },
  { name: "Kodi", key: "kodi", type: "", comment: "" },
  { name: "exwm", key: "exwm", type: "", comment: "" },
  { name: "Openbox", key: "openbox", type: "", comment: "" },
  { name: "Sway", key: "sway", type: "", comment: "" },
]);
assertEquals(greeter.users, [
  {
    display_name: "John Doe",
    username: "johnd",
    image: "",
    language: "en_US.UTF-8",
    home_directory: "",
    session: "",
  },
  {
    display_name: "Zayn Chen",
    username: "zaync",
    image: "",
    language: "zh_CN.UTF-8",
    session: "hyprland",
    home_directory: "",
  },
]);
assert(greeter.can_hibernate);
assert(greeter.can_restart);
assert(greeter.can_shutdown);
assert(greeter.can_suspend);
assert(greeter.hibernate);
assert(greeter.restart);
assert(greeter.shutdown);
assert(greeter.suspend);
assertEquals(greeter.authentication_user, null);
assertFalse(greeter.in_authentication);
assertFalse(greeter.is_authenticated);
assert(greeter.layout = "en");
assert(greeter.cancel_authentication());
assert(greeter.authenticate("test"));
assert(greeter.respond("test"));
assert(greeter.start_session("hyprland"));
