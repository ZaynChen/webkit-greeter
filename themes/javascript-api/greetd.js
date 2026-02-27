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

// greetd/mod.ts
var Greeter = class {
  show_prompt;
  show_message;
  authentication_complete;
  constructor() {
    this.show_prompt = new Signal();
    this.show_message = new Signal();
    this.authentication_complete = new Signal();
  }
  #sendRequest(method, args) {
    return sendRequest("greeter", method, args);
  }
  get authentication_user() {
    return this.#sendRequest("authentication_user");
  }
  get can_hibernate() {
    return this.#sendRequest("can_hibernate");
  }
  get can_restart() {
    return this.#sendRequest("can_restart");
  }
  get can_shutdown() {
    return this.#sendRequest("can_shutdown");
  }
  get can_suspend() {
    return this.#sendRequest("can_suspend");
  }
  get in_authentication() {
    return this.#sendRequest("in_authentication");
  }
  get is_authenticated() {
    return this.#sendRequest("is_authenticated");
  }
  get language() {
    return this.#sendRequest("language");
  }
  get languages() {
    return this.#sendRequest("languages");
  }
  get layout() {
    return this.#sendRequest("layout");
  }
  set layout(layout) {
    this.#sendRequest("layout", [
      layout
    ]);
  }
  get layouts() {
    return this.#sendRequest("layouts");
  }
  get sessions() {
    return this.#sendRequest("sessions");
  }
  get users() {
    return this.#sendRequest("users");
  }
  hibernate() {
    return this.#sendRequest("hibernate");
  }
  restart() {
    return this.#sendRequest("restart");
  }
  shutdown() {
    return this.#sendRequest("shutdown");
  }
  suspend() {
    return this.#sendRequest("suspend");
  }
  authenticate(username) {
    return this.#sendRequest("authenticate", [
      username
    ]);
  }
  respond(password = null) {
    return this.#sendRequest("respond", [
      password
    ]);
  }
  cancel_authentication() {
    return this.#sendRequest("cancel_authentication");
  }
  start_session(session) {
    return this.#sendRequest("start_session", [
      session
    ]);
  }
};
globalThis.greeter = new Greeter();
