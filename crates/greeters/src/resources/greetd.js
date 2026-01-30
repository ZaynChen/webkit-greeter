// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

const send_request = (target, method, args) => {
  const request = {
    target,
    method,
    args,
  };
  return window.send_request(request);
};

class Language {
  code;
  name;
  territory;
  constructor({ code, name, territory }) {
    this.code = code;
    this.name = name;
    this.territory = territory;
  }
}

class Layout {
  description;
  name;
  short_description;
  constructor({ description, name, short_description }) {
    this.description = description;
    this.name = name;
    this.short_description = short_description;
  }
}

class Session {
  comment;
  key;
  name;
  type;
  constructor({ comment, key, name, type }) {
    this.comment = comment;
    this.key = key;
    this.name = name;
    this.type = type;
  }
}

class User {
  background;
  display_name;
  home_directory;
  image;
  language;
  // layout;
  // layouts;
  // logged_in;
  session;
  username;
  constructor(user_info) {
    Object.keys(user_info).forEach((key) => {
      this[key] = user_info[key];
    });
  }
}

class Signal {
  #callbacks = [];
  connect(callback) {
    if (typeof callback === "function") {
      this.#callbacks.push(callback);
    }
  }
  disconnect(callback) {
    if (typeof callback === "function") {
      this.#callbacks = this.#callbacks.filter((cb) => cb !== callback);
    }
  }
  _emit(...args) {
    this.#callbacks.forEach((cb) => {
      cb(...args);
    });
  }
}

class Greeter {
  authentication_complete;
  show_prompt;
  show_message;
  constructor() {
    this.authentication_complete = new Signal();
    this.show_prompt = new Signal();
    this.show_message = new Signal();
  }
  #send_request(method, args) {
    return send_request("greeter", method, args);
  }
  get authentication_user() {
    return this.#send_request("authentication_user");
  }
  get can_hibernate() {
    return this.#send_request("can_hibernate");
  }
  get can_restart() {
    return this.#send_request("can_restart");
  }
  get can_shutdown() {
    return this.#send_request("can_shutdown");
  }
  get can_suspend() {
    return this.#send_request("can_suspend");
  }
  get in_authentication() {
    return this.#send_request("in_authentication");
  }
  get is_authenticated() {
    return this.#send_request("is_authenticated");
  }
  get language() {
    return new Language(this.#send_request("language"));
  }
  get languages() {
    return this.#send_request("languages").map((l) => new Language(l));
  }
  get layout() {
    return new Layout(this.#send_request("layout"));
  }
  set layout(value) {
    let val = "string" === typeof value ? value : value.name;
    this.#send_request("layout", [val]);
  }
  get layouts() {
    return this.#send_request("layouts").map((l) => new Layout(l));
  }
  get sessions() {
    return this.#send_request("sessions").map((s) => new Session(s));
  }
  get users() {
    return this.#send_request("users").map((u) => new User(u));
  }
  hibernate() {
    return this.#send_request("hibernate");
  }
  restart() {
    return this.#send_request("restart");
  }
  shutdown() {
    return this.#send_request("shutdown");
  }
  suspend() {
    return this.#send_request("suspend");
  }
  authenticate(username = null) {
    return this.#send_request("authenticate", [username]);
  }
  respond(password) {
    return this.#send_request("respond", [password]);
  }
  cancel_authentication() {
    return this.#send_request("cancel_authentication");
  }
  start_session(session) {
    return this.#send_request("start_session", [session]);
  }
}

class GreeterComm {
  #send_request(method, args) {
    return send_request("greeter_comm", method, args);
  }
  get window_metadata() {
    return this.#send_request("window_metadata");
  }
  broadcast(data) {
    return this.#send_request("broadcast", [data]);
  }
  _emit(data) {
    const broadcast_event = new Event("GreeterBroadcastEvent");
    broadcast_event.window = null;
    broadcast_event.data = data;
    dispatchEvent(broadcast_event);
  }
}

class GreeterConfig {
  #send_request(method) {
    return send_request("greeter_config", method);
  }
  get branding() {
    return this.#send_request("branding");
  }
  get greeter() {
    return this.#send_request("greeter");
  }
  get layouts() {
    return send_request("greeter", "layouts").map((l) => new Layout(l));
  }
}

class ThemeUtils {
  #time_language;
  constructor(time_language = "") {
    this.#time_language = time_language;
  }
  #send_request(method, args) {
    return send_request("theme_utils", method, args);
  }

  /**
   * Returns the contents of directory found at `path` provided that the (normalized) `path`
   * meets at least one of the following conditions:
   *   * Is located within the greeter themes' root directory.
   *   * Has been explicitly allowed in the greeter's config file.
   *   * Is located within the greeter's shared data directory (`/var/lib/lightdm-data`).
   *   * Is located in `/tmp`.
   *
   * @param {string}              path        The abs path to desired directory.
   * @param {boolean}             only_images Include only images in the results. Default `true`.
   * @param {function(string[])}  callback    Callback function to be called with the result.
   */
  dirlist(path, only_image = true, callback) {
    if ("" === path || "string" !== typeof path) {
      console.error(
        "[ERROR] theme_utils.dirlist(): path must be a non-empty string!",
      );
      return callback([]);
    } else if (null !== path.match(/^[^/].+/)) {
      console.error("[ERROR] theme_utils.dirlist(): path must be absolute!");
      return callback([]);
    }

    if (null !== path.match(/\/\.+(?=\/)/)) {
      // No special directory names allowed (eg ../../)
      path = path.replace(/\/\.+(?=\/)/g, "");
    }

    try {
      const result = this.#send_request("dirlist", [path, only_image]);
      callback(result);
      return result;
    } catch (err) {
      console.error(`[ERROR] theme_utils.dirlist(): ${err}`);
      return callback([]);
    }
  }
  get_current_localized_date() {
    const locales = [];
    const time_language = this.#time_language;
    if ("" !== time_language) {
      locales.push(time_language);
    }
    return new Intl.DateTimeFormat(locales, {
      day: "2-digit",
      month: "2-digit",
      year: "2-digit",
    }).format(new Date());
  }
  /**
   * Get the current time in a localized format. Time format and language are auto-detected
   * by default, but can be set manually in the greeter config file.
   *   * `language` defaults to the system's language, but can be set manually in the config file.
   *   * When `time_format` config file option has a valid value, time will be formatted
   *     according to that value.
   *   * When `time_format` does not have a valid value, the time format will be `LT`
   *     which is `1:00 PM` or `13:00` depending on the system's locale.
   *
   * @return {string} The current localized time.
   */
  get_current_localized_time() {
    const locales = [];
    const time_language = this.#time_language;
    if ("" !== time_language) {
      locales.push(time_language);
    }
    return new Intl.DateTimeFormat(locales, {
      hour: "2-digit",
      minute: "2-digit",
    }).format(new Date());
  }
}

window.greeter_comm = new GreeterComm();
window.greeter_config = new GreeterConfig();
window.greeter = new Greeter();
window.theme_utils = new ThemeUtils(
  window.greeter_config.greeter.time_language,
);
window.lightdm = window.greeter;
window._ready_event = new Event("GreeterReady");
window.dispatch_ready_event = () => dispatchEvent(window._ready_event);
