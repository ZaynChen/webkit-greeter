// common/api.ts
function sendRequest(target, method, args = []) {
  return globalThis.send_request({
    target,
    method,
    args: JSON.stringify(args)
  });
}

// common/mod.ts
var GreeterComm = class {
  broadcast(data) {
    return sendRequest("greeter_comm", "broadcast", [
      data
    ]);
  }
  _emit(data) {
    return dispatchEvent(new CustomEvent("GreeterBroadcastEvent", {
      detail: data
    }));
  }
};
var GreeterConfig = class {
  #sendRequest(method) {
    return sendRequest("greeter_config", method);
  }
  get branding() {
    return this.#sendRequest("branding");
  }
  get greeter() {
    return this.#sendRequest("greeter");
  }
  get layouts() {
    return sendRequest("greeter", "layouts");
  }
};
var ThemeUtils = class {
  async dirlist(path, only_image = true) {
    if ("" === path || "string" !== typeof path) {
      console.error("[ERROR] theme_utils.dirlist(): path must be a non-empty string!");
      return [];
    } else if (null !== path.match(/^[^/].+/)) {
      console.error("[ERROR] theme_utils.dirlist(): path must be absolute!");
      return [];
    }
    if (null !== path.match(/\/\.+(?=\/)/)) {
      path = path.replace(/\/\.+(?=\/)/g, "");
    }
    try {
      return await new Promise((resolve) => resolve(sendRequest("theme_utils", "dirlist", [
        path,
        only_image
      ])));
    } catch (err) {
      console.error(`[ERROR] theme_utils.dirlist(): ${err}`);
      return [];
    }
  }
};
globalThis.greeter_comm = new GreeterComm();
globalThis.greeter_config = new GreeterConfig();
globalThis.theme_utils = new ThemeUtils();
globalThis.dispatch_ready_event = () => dispatchEvent(new Event("GreeterReady"));
var Signal = class {
  #callbacks = [];
  connect(callback) {
    this.#callbacks.push(callback);
  }
  disconnect(callback) {
    this.#callbacks = this.#callbacks.filter((cb) => cb !== callback);
  }
  _emit(...args) {
    this.#callbacks.forEach((cb) => {
      cb(...args);
    });
  }
};

// lightdm/mod.ts
var Greeter = class {
  show_prompt;
  show_message;
  authentication_complete;
  autologin_timer_expired;
  constructor() {
    this.show_prompt = new Signal();
    this.show_message = new Signal();
    this.authentication_complete = new Signal();
    this.autologin_timer_expired = new Signal();
  }
  #sendRequest(method, args) {
    return sendRequest("greeter", method, args);
  }
  /**
   * The username of the user being authenticated or {@link null}
   * if there is no authentication in progress.
   */
  get authentication_user() {
    return this.#sendRequest("authentication_user");
  }
  /**
   * Whether or not the guest account should be automatically logged
   * into when the timer expires.
   */
  get autologin_guest() {
    return this.#sendRequest("autologin_guest");
  }
  /**
   * The number of seconds to wait before automatically logging in.
   */
  get autologin_timeout() {
    return this.#sendRequest("autologin_timeout");
  }
  /**
   * The username with which to automatically log in when the timer expires.
   */
  get autologin_user() {
    return this.#sendRequest("autologin_user");
  }
  /**
   * Whether or not the greeter can make the system hibernate.
   */
  get can_hibernate() {
    return this.#sendRequest("can_hibernate");
  }
  /**
   * Whether or not the greeter can make the system restart.
   */
  get can_restart() {
    return this.#sendRequest("can_restart");
  }
  /**
   * Whether or not the greeter can make the system shutdown.
   */
  get can_shutdown() {
    return this.#sendRequest("can_shutdown");
  }
  /**
   * Whether or not the greeter can make the system suspend/sleep.
   */
  get can_suspend() {
    return this.#sendRequest("can_suspend");
  }
  /**
   * The name of the default session.
   */
  get default_session() {
    return this.#sendRequest("default_session");
  }
  /**
   * Whether or not guest sessions are supported.
   */
  get has_guest_account() {
    return this.#sendRequest("has_guest_account");
  }
  /**
   * Whether or not user accounts should be hidden.
   */
  get hide_users_hint() {
    return this.#sendRequest("hide_users_hint");
  }
  /**
   * The system's hostname.
   */
  get hostname() {
    return this.#sendRequest("hostname");
  }
  /**
   * Whether or not the greeter is in the process of authenticating.
   */
  get in_authentication() {
    return this.#sendRequest("in_authentication");
  }
  /**
   * Whether or not the greeter has successfully authenticated.
   */
  get is_authenticated() {
    return this.#sendRequest("is_authenticated");
  }
  /**
   * The current language or {@link null} if no language.
   */
  get language() {
    return this.#sendRequest("language");
  }
  /**
   * A list of languages to present to the user.
   */
  get languages() {
    return this.#sendRequest("languages");
  }
  /**
   * The currently active layout for the selected user.
   */
  get layout() {
    return this.#sendRequest("layout");
  }
  /**
   * Set the active layout for the selected user.
   */
  set layout(value) {
    this.#sendRequest("layout", [
      value
    ]);
  }
  /**
   * A list of keyboard layouts to present to the user.
   */
  get layouts() {
    return this.#sendRequest("layouts");
  }
  /**
   * Whether or not the greeter was started as a lock screen.
   */
  get lock_hint() {
    return this.#sendRequest("lock_hint");
  }
  /**
   * The available remote sessions.
   */
  get remote_sessions() {
    return this.#sendRequest("remote_sessions");
  }
  /**
   * Whether or not the guest account should be selected by default.
   */
  get select_guest_hint() {
    return this.#sendRequest("select_guest_hint");
  }
  /**
   * The username to select by default.
   */
  get select_user_hint() {
    return this.#sendRequest("select_user_hint");
  }
  /**
   * List of available sessions.
   */
  get sessions() {
    return this.#sendRequest("sessions");
  }
  /**
   * Check if a manual login option should be shown. If {@link true}, the theme should
   * provide a way for a username to be entered manually. Otherwise, themes that show
   * a user list may limit logins to only those users.
   */
  get show_manual_login_hint() {
    return this.#sendRequest("show_manual_login_hint");
  }
  /**
   * Check if a remote login option should be shown. If {@link true}, the theme should provide
   * a way for a user to log into a remote desktop server.
   */
  get show_remote_login_hint() {
    return this.#sendRequest("show_remote_login_hint");
  }
  /**
   * List of available users.
   */
  get users() {
    return this.#sendRequest("users");
  }
  get shared_data_directory() {
    return this.#sendRequest("shared_data_directory");
  }
  /**
   * Starts the authentication procedure for a user.
   * @arg {String|null} username A username or {@link null} to prompt for a username.
   */
  authenticate(username = null) {
    return this.#sendRequest("authenticate", [
      username
    ]);
  }
  /**
   * Starts the authentication procedure for the guest user.
   */
  authenticate_as_guest() {
    return this.#sendRequest("authenticate_as_guest");
  }
  /**
   * Cancel the user authentication that is currently in progress.
   */
  cancel_authentication() {
    return this.#sendRequest("cancel_authentication");
  }
  /**
   * Cancel the automatic login.
   */
  cancel_autologin() {
    return this.#sendRequest("cancel_autologin");
  }
  /**
   * Triggers the system to hibernate.
   * @returns {boolean} {@link true} if hibernation initiated, otherwise {@link false}
   */
  hibernate() {
    return this.#sendRequest("hibernate");
  }
  /**
   * Provide a response to a prompt.
   */
  respond(response) {
    return this.#sendRequest("respond", [
      response
    ]);
  }
  /**
   * Triggers the system to restart.
   * @returns {boolean} {@link true} if restart initiated, otherwise {@link false}
   */
  restart() {
    return this.#sendRequest("restart");
  }
  /**
   * Set the language for the currently authenticated user.
   * @arg {string} language The language in the form of a locale specification (e.g. 'de_DE.UTF-8')
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  set_language(language) {
    return this.#sendRequest("set_language", [
      language
    ]);
  }
  /**
   * Triggers the system to shutdown.
   * @returns {boolean} {@link true} if shutdown initiated, otherwise {@link false}
   */
  shutdown() {
    return this.#sendRequest("shutdown");
  }
  /**
   * Start a session for the authenticated user.
   * @arg {String|null} session The session to log into or {@link null} to use the default.
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  start_session(session) {
    return this.#sendRequest("start_session", [
      session
    ]);
  }
  /**
   * Triggers the system to suspend/sleep.
   * @returns {boolean} {@link true} if suspend/sleep initiated, otherwise {@link false}
   */
  suspend() {
    return this.#sendRequest("suspend");
  }
};
globalThis.greeter = new Greeter();
globalThis.lightdm = globalThis.greeter;
