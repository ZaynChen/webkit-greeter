// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

const send_request = (target, method, args = []) => {
  const request = {
    target,
    method,
    args,
  };
  return globalThis.send_request(request);
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
  display_name;
  home_directory;
  image;
  language;
  logged_in;
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
    const language = this.#send_request("language");
    return language && new Language(language);
  }
  get languages() {
    return this.#send_request("languages")?.map((l) => new Language(l));
  }
  get layout() {
    const layout = this.#send_request("layout");
    return layout && new Layout(layout);
  }
  set layout(value) {
    let val = "string" === typeof value ? value : value.name;
    this.#send_request("layout", [val]);
  }
  get layouts() {
    return this.#send_request("layouts")?.map((l) => new Layout(l));
  }
  get sessions() {
    return this.#send_request("sessions")?.map((s) => new Session(s));
  }
  get users() {
    return this.#send_request("users")?.map((u) => new User(u));
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
  /**
   * Starts the authentication procedure for a user.
   *
   * @arg {String} username A username
   */
  authenticate(username) {
    return this.#send_request("authenticate", [username]);
  }
  /**
   * Provide a response to a prompt.
   * @arg {string} password
   * @returns {false} session is not exist or already authenticated, or can
   *                 not send request to greetd
   *  otherwise {true} if send request to greetd seccessful
   */
  respond(password = null) {
    return this.#send_request("respond", [password]);
  }
  /**
   * Cancel the user authentication that is currently in progress.
   */
  cancel_authentication() {
    return this.#send_request("cancel_authentication");
  }
  /**
   * Start a session for the authenticated user.
   * @arg {String} session The session to log into
   * @return {@link false} failed to start session
   *   otherwise exit the webkit-greeter and wait for user session
   *      and this function will not return
   */
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
    dispatchEvent(
      new CustomEvent("GreeterBroadcastEvent", {
        detail: data,
      }),
    );
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
    return send_request("greeter", "layouts")?.map((l) => new Layout(l));
  }
}

class ThemeUtils {
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
   */
  async dirlist(path, only_image = true) {
    if ("" === path || "string" !== typeof path) {
      console.error(
        "[ERROR] theme_utils.dirlist(): path must be a non-empty string!",
      );
      return [];
    } else if (null !== path.match(/^[^/].+/)) {
      console.error("[ERROR] theme_utils.dirlist(): path must be absolute!");
      return [];
    }

    if (null !== path.match(/\/\.+(?=\/)/)) {
      // No special directory names allowed (eg ../../)
      path = path.replace(/\/\.+(?=\/)/g, "");
    }

    try {
      return this.#send_request("dirlist", [path, only_image]);
    } catch (err) {
      console.error(`[ERROR] theme_utils.dirlist(): ${err}`);
      return [];
    }
  }
}

globalThis.greeter_comm = new GreeterComm();
globalThis.greeter_config = new GreeterConfig();
globalThis.greeter = new Greeter();
globalThis.theme_utils = new ThemeUtils();
globalThis.lightdm = globalThis.greeter;
globalThis._ready_event = new Event("GreeterReady");
globalThis.dispatch_ready_event = () => dispatchEvent(globalThis._ready_event);
