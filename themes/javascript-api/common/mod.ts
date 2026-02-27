// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

import type {
  GreeterConfigBranding,
  GreeterConfigGreeter,
  GreeterRequestMethod,
  Layout,
} from "../types.d.ts";

import { sendRequest } from "./api.ts";

class GreeterComm {
  broadcast<T>(data: T) {
    return sendRequest("greeter_comm", "broadcast", [data]);
  }
  _emit<T>(data: T) {
    return dispatchEvent(
      new CustomEvent("GreeterBroadcastEvent", {
        detail: data,
      }),
    );
  }
}

class GreeterConfig {
  #sendRequest(method: GreeterRequestMethod["greeter_config"]) {
    return sendRequest("greeter_config", method);
  }
  get branding(): GreeterConfigBranding {
    return this.#sendRequest("branding") as GreeterConfigBranding;
  }
  get greeter(): GreeterConfigGreeter {
    return this.#sendRequest("greeter") as GreeterConfigGreeter;
  }
  get layouts(): Layout[] {
    return sendRequest("greeter", "layouts") as Layout[];
  }
}

class ThemeUtils {
  async dirlist(path: string, only_image: boolean = true): Promise<string[]> {
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
      return await new Promise((resolve) =>
        resolve(
          sendRequest("theme_utils", "dirlist", [path, only_image]) as string[],
        )
      );
    } catch (err) {
      console.error(`[ERROR] theme_utils.dirlist(): ${err}`);
      return [];
    }
  }
}

globalThis.greeter_comm = new GreeterComm();
globalThis.greeter_config = new GreeterConfig();
globalThis.theme_utils = new ThemeUtils();
globalThis.dispatch_ready_event = () =>
  dispatchEvent(new Event("GreeterReady"));

export class Signal {
  #callbacks: ((...args: string[]) => void)[] = [];
  connect(callback: (...args: string[]) => void) {
    this.#callbacks.push(callback);
  }
  disconnect(callback: (...args: string[]) => void) {
    this.#callbacks = this.#callbacks.filter((cb) => cb !== callback);
  }
  _emit(...args: string[]) {
    this.#callbacks.forEach((cb) => {
      cb(...args);
    });
  }
}
