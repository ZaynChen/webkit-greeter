// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

globalThis.send_request = ({ target, method, args }) => {
  switch (target) {
    case "greeter":
      return greeterHandler(method, args);
    case "greeter_config":
      return greeterConfigHandler(method);
    case "greeter_comm":
      return greeterCommHandler(method, args);
    case "theme_utils":
      return themeUtilsHandler(method, args);
    default:
      console.log(`unimplement target "${target}"`);
      return undefined;
  }
};

function greeterHandler(
  method: string,
  param: string,
) {
  if (param === "[]") {
    switch (method) {
      case "can_hibernate":
        return true;
      case "can_restart":
        return true;
      case "can_shutdown":
        return true;
      case "can_suspend":
        return true;
      case "hibernate":
        return true;
      case "restart":
        return true;
      case "shutdown":
        return true;
      case "suspend":
        return true;
      case "language":
        return {
          code: "en_US.utf8",
          name: "American English",
          territory: "United States",
        };
      case "languages":
        return [
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
        ];
      case "layout":
        return {
          name: "us",
          description: "English (US)",
          short_description: "en",
        };
      case "layouts":
        return [
          {
            name: "us",
            description: "English (US)",
            short_description: "en",
          },
        ];
      case "sessions":
        return [
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
        ];
      case "users":
        return [
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
        ];
      case "authentication_user":
        return null;
      case "in_authentication":
        return false;
      case "is_authenticated":
        return false;
      case "cancel_authentication":
        return true;
      default:
        console.log(`unimplement method "${method}(${param})"`);
        return undefined;
    }
  } else {
    switch (method) {
      case "layout":
        return true;
      case "authenticate":
        return true;
      case "respond":
        return true;
      case "start_session":
        return true;
      default:
        console.log(`unimplement method "${method}(${param})"`);
        return undefined;
    }
  }
}

function greeterConfigHandler(
  method: string,
) {
  switch (method) {
    case "branding":
      return {
        background_images_dir: "/user/share/backgrounds",
        logo_image: "./assets/logo.png",
        user_image: "./assets/default_user.png",
      };
    case "greeter":
      return {
        debug_mode: true,
        detect_theme_errors: true,
        screensaver_timeout: 300,
        secure_mode: true,
        theme: "litarvan",
        icon_theme: null,
        time_language: null,
      };
    default:
      console.log(`unimplement method "${method}"`);
      return undefined;
  }
}
function greeterCommHandler(method: string, param: string) {
  if (method === "broadcast") {
    globalThis.greeter_comm._emit(JSON.parse(param));
    return null;
  }
  console.log(`unimplement method "${method}(${param})"`);
  return undefined;
}
function themeUtilsHandler(
  method: string,
  param: string,
) {
  if (method === "dirlist") {
    const [path, only_images]: [string, boolean] = JSON.parse(param);
    if (path.trim().match(/^[|/|\./]$/) || !path.trim().startsWith("/")) {
      return [];
    }
    if (only_images) {
      return [
        "/usr/share/backgrounds/archlinux/archbtw.png",
        "/usr/share/backgrounds/archlinux/awesome.png",
      ];
    } else {
      return [
        "/usr/share/backgrounds/archlinux/archlinux.stw",
        "/usr/share/backgrounds/archlinux/archbtw.png",
        "/usr/share/backgrounds/archlinux/awesome.png",
      ];
    }
  }
  console.log(`unimplement method "${method}(${param})"`);
  return undefined;
}

import "@scope/greetd";
