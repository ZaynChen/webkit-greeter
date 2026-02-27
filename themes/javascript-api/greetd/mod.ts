// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

import type {
  GreeterRequestMethod,
  Language,
  Layout,
  Session,
  User,
} from "../types.d.ts";

import { sendRequest } from "@scope/common/api";
import { Signal } from "@scope/common";

class Greeter {
  show_prompt;
  show_message;
  authentication_complete;
  constructor() {
    this.show_prompt = new Signal();
    this.show_message = new Signal();
    this.authentication_complete = new Signal();
  }
  #sendRequest(
    method: GreeterRequestMethod["greeter"],
    args?: (string | null)[],
  ) {
    return sendRequest("greeter", method, args);
  }
  get authentication_user(): string | null {
    return this.#sendRequest("authentication_user") as string | null;
  }
  get can_hibernate(): boolean {
    return this.#sendRequest("can_hibernate") as boolean;
  }
  get can_restart(): boolean {
    return this.#sendRequest("can_restart") as boolean;
  }
  get can_shutdown(): boolean {
    return this.#sendRequest("can_shutdown") as boolean;
  }
  get can_suspend(): boolean {
    return this.#sendRequest("can_suspend") as boolean;
  }
  get in_authentication(): boolean {
    return this.#sendRequest("in_authentication") as boolean;
  }
  get is_authenticated(): boolean {
    return this.#sendRequest("is_authenticated") as boolean;
  }
  get language(): Language | null {
    return this.#sendRequest("language") as Language | null;
  }
  get languages(): Language[] {
    return this.#sendRequest("languages") as Language[];
  }
  get layout(): Layout {
    return this.#sendRequest("layout") as Layout;
  }
  set layout(layout: string) {
    this.#sendRequest("layout", [layout]);
  }
  get layouts(): Layout[] {
    return this.#sendRequest("layouts") as Layout[];
  }
  get sessions(): Session[] {
    return this.#sendRequest("sessions") as Session[];
  }
  get users(): User[] {
    return this.#sendRequest("users") as User[];
  }
  hibernate(): boolean {
    return this.#sendRequest("hibernate") as boolean;
  }
  restart(): boolean {
    return this.#sendRequest("restart") as boolean;
  }
  shutdown(): boolean {
    return this.#sendRequest("shutdown") as boolean;
  }
  suspend(): boolean {
    return this.#sendRequest("suspend") as boolean;
  }
  authenticate(username: string): boolean {
    return this.#sendRequest("authenticate", [username]) as boolean;
  }
  respond(password: string | null = null): boolean {
    return this.#sendRequest("respond", [password]) as boolean;
  }
  cancel_authentication(): boolean {
    return this.#sendRequest("cancel_authentication") as boolean;
  }
  start_session(session: string): boolean {
    return this.#sendRequest("start_session", [session]) as boolean;
  }
}

globalThis.greeter = new Greeter();
