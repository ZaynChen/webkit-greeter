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

class LightDMLanguage {
  code;
  name;
  territory;
  constructor({ code, name, territory }) {
    this.code = code;
    this.name = name;
    this.territory = territory;
  }
}

class LightDMLayout {
  description;
  name;
  short_description;
  constructor({ description, name, short_description }) {
    this.description = description;
    this.name = name;
    this.short_description = short_description;
  }
}

class LightDMSession {
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

class LightDMUser {
  background;
  display_name;
  home_directory;
  image;
  language;
  layout;
  layouts;
  logged_in;
  session;
  username;
  constructor(user_info) {
    Object.keys(user_info).forEach((key) => {
      this[key] = user_info[key];
    });
  }
}

class LightDMSignal {
  _name;
  _callbacks;
  constructor(name) {
    this._name = name;
    this._callbacks = [];
  }
  connect(callback) {
    if (typeof callback === "function") {
      this._callbacks.push(callback);
    }
  }
  disconnect(callback) {
    if (typeof callback === "function") {
      this._callbacks = this._callbacks.filter((_cb) => _cb !== callback);
    }
  }
  _emit(...args) {
    this._callbacks.forEach((callback) => {
      callback(...args);
    });
  }
}

class LightDMGreeter {
  authentication_complete;
  autologin_timer_expired;
  show_prompt;
  show_message;

  constructor() {
    this.authentication_complete = new LightDMSignal(
      "authentication_complete ",
    );
    this.autologin_timer_expired = new LightDMSignal("autologin_timer_expired");
    this.show_prompt = new LightDMSignal("show_prompt");
    this.show_message = new LightDMSignal("show_message");
  }

  #send_request(method, args) {
    return send_request("greeter", method, args);
  }

  /**
   * The username of the user being authenticated or {@link null}
   * if there is no authentication in progress.
   * @type {string|null}
   * @readonly
   */
  get authentication_user() {
    return this.#send_request("authentication_user");
  }

  /**
   * Whether or not the guest account should be automatically logged
   * into when the timer expires.
   * @type {boolean}
   * @readonly
   */
  get autologin_guest() {
    return this.#send_request("autologin_guest");
  }

  /**
   * The number of seconds to wait before automatically logging in.
   * @type {number}
   * @readonly
   */
  get autologin_timeout() {
    return this.#send_request("autologin_timeout");
  }

  /**
   * The username with which to automatically log in when the timer expires.
   * @type {string}
   * @readonly
   */
  get autologin_user() {
    return this.#send_request("autologin_user");
  }

  /**
   * Whether or not the greeter can make the system hibernate.
   * @type {boolean}
   * @readonly
   */
  get can_hibernate() {
    return this.#send_request("can_hibernate");
  }

  /**
   * Whether or not the greeter can make the system restart.
   * @type {boolean}
   * @readonly
   */
  get can_restart() {
    return this.#send_request("can_restart");
  }

  /**
   * Whether or not the greeter can make the system shutdown.
   * @type {boolean}
   * @readonly
   */
  get can_shutdown() {
    return this.#send_request("can_shutdown");
  }

  /**
   * Whether or not the greeter can make the system suspend/sleep.
   * @type {boolean}
   * @readonly
   */
  get can_suspend() {
    return this.#send_request("can_suspend");
  }

  /**
   * The name of the default session.
   * @type {string}
   * @readonly
   */
  get default_session() {
    return this.#send_request("default_session");
  }

  /**
   * Whether or not guest sessions are supported.
   * @type {boolean}
   * @readonly
   */
  get has_guest_account() {
    return this.#send_request("has_guest_account");
  }

  /**
   * Whether or not user accounts should be hidden.
   * @type {boolean}
   * @readonly
   */
  get hide_users_hint() {
    return this.#send_request("hide_users_hint");
  }

  /**
   * The system's hostname.
   * @type {string}
   * @readonly
   */
  get hostname() {
    return this.#send_request("hostname");
  }

  /**
   * Whether or not the greeter is in the process of authenticating.
   * @type {boolean}
   * @readonly
   */
  get in_authentication() {
    return this.#send_request("in_authentication");
  }

  /**
   * Whether or not the greeter has successfully authenticated.
   * @type {boolean}
   * @readonly
   */
  get is_authenticated() {
    return this.#send_request("is_authenticated");
  }

  /**
   * The current language or {@link null} if no language.
   * @type {LightDM.Language|null}
   * @readonly
   */
  get language() {
    return new LightDMLanguage(this.#send_request("language"));
  }

  /**
   * A list of languages to present to the user.
   * @type {LightDM.Language[]}
   * @readonly
   */
  get languages() {
    return this.#send_request("languages").map((l) => new LightDMLanguage(l));
  }

  /**
   * The currently active layout for the selected user.
   * @type {LightDM.Layout}
   */
  get layout() {
    return new LightDMLayout(this.#send_request("layout"));
  }

  /**
   * Set the active layout for the selected user.
   * @param {LightDM.Layout} value
   */
  set layout(value) {
    this.#send_request("layout", [value]);
  }

  /**
   * A list of keyboard layouts to present to the user.
   * @type {LightDM.Layout[]}
   * @readonly
   */
  get layouts() {
    return this.#send_request("layouts").map((l) => new LightDMLayout(l));
  }

  /**
   * Whether or not the greeter was started as a lock screen.
   * @type {boolean}
   * @readonly
   */
  get lock_hint() {
    return this.#send_request("lock_hint");
  }

  /**
   * The available remote sessions.
   * @type {LightDM.Session[]}
   * @readonly
   */
  get remote_sessions() {
    return this.#send_request("remote_sessions").map(
      (s) => new LightDMSession(s),
    );
  }

  /**
   * Whether or not the guest account should be selected by default.
   * @type {boolean}
   * @readonly
   */
  get select_guest_hint() {
    return this.#send_request("select_guest_hint");
  }

  /**
   * The username to select by default.
   * @type {string}
   * @readonly
   */
  get select_user_hint() {
    return this.#send_request("select_user_hint");
  }

  /**
   * List of available sessions.
   * @type {LightDM.Session[]}
   * @readonly
   */
  get sessions() {
    return this.#send_request("sessions").map((s) => new LightDMSession(s));
  }

  /**
   * Check if a manual login option should be shown. If {@link true}, the theme should
   * provide a way for a username to be entered manually. Otherwise, themes that show
   * a user list may limit logins to only those users.
   * @type {string}
   * @readonly
   */
  get show_manual_login_hint() {
    return this.#send_request("show_manual_login_hint");
  }

  /**
   * Check if a remote login option should be shown. If {@link true}, the theme should provide
   * a way for a user to log into a remote desktop server.
   * @type {string}
   * @readonly
   * @internal
   */
  get show_remote_login_hint() {
    return this.#send_request("show_remote_login_hint");
  }

  /**
   * List of available users.
   * @type {LightDM.User[]}
   * @readonly
   */
  get users() {
    return this.#send_request("users").map((u) => new LightDMUser(u));
  }

  get shared_data_directory() {
    return this.#send_request("shared_data_directory");
  }

  /**
   * Starts the authentication procedure for a user.
   *
   * @arg {String|null} username A username or {@link null} to prompt for a username.
   */
  authenticate(username = null) {
    return this.#send_request("authenticate", [username]);
  }

  /**
   * Starts the authentication procedure for the guest user.
   */
  authenticate_as_guest() {
    return this.#send_request("authenticate_as_guest");
  }

  /**
   * Cancel the user authentication that is currently in progress.
   */
  cancel_authentication() {
    return this.#send_request("cancel_authentication");
  }

  /**
   * Cancel the automatic login.
   */
  cancel_autologin() {
    return this.#send_request("cancel_autologin");
  }

  /**
   * Triggers the system to hibernate.
   * @returns {boolean} {@link true} if hibernation initiated, otherwise {@link false}
   */
  hibernate() {
    return this.#send_request("hibernate");
  }

  /**
   * Provide a response to a prompt.
   * @arg {string} response
   */
  respond(password) {
    return this.#send_request("respond", [password]);
  }

  /**
   * Triggers the system to restart.
   * @returns {boolean} {@link true} if restart initiated, otherwise {@link false}
   */
  restart() {
    return this.#send_request("restart");
  }

  /**
   * Set the language for the currently authenticated user.
   * @arg {string} language The language in the form of a locale specification (e.g. 'de_DE.UTF-8')
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  set_language(value) {
    return this.#send_request("set_language", [value]);
  }

  /**
   * Triggers the system to shutdown.
   * @returns {boolean} {@link true} if shutdown initiated, otherwise {@link false}
   */
  shutdown() {
    return this.#send_request("shutdown");
  }

  /**
   * Start a session for the authenticated user.
   * @arg {String|null} session The session to log into or {@link null} to use the default.
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  start_session(session) {
    return this.#send_request("start_session", [session]);
  }

  /**
   * Triggers the system to suspend/sleep.
   * @returns {boolean} {@link true} if suspend/sleep initiated, otherwise {@link false}
   */
  suspend() {
    return this.#send_request("suspend");
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
    return [];
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
