// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

import type {
  Greeter as GreeterClass,
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
  autologin_timer_expired;
  constructor() {
    this.show_prompt = new Signal();
    this.show_message = new Signal();
    this.authentication_complete = new Signal();
    this.autologin_timer_expired = new Signal();
  }
  #sendRequest(
    method: GreeterRequestMethod["greeter"],
    args?: (string | null)[],
  ) {
    return sendRequest("greeter", method, args);
  }

  /**
   * The username of the user being authenticated or {@link null}
   * if there is no authentication in progress.
   */
  get authentication_user(): string | null {
    return this.#sendRequest("authentication_user") as string | null;
  }

  /**
   * Whether or not the guest account should be automatically logged
   * into when the timer expires.
   */
  get autologin_guest(): boolean {
    return this.#sendRequest("autologin_guest") as boolean;
  }

  /**
   * The number of seconds to wait before automatically logging in.
   */
  get autologin_timeout(): number {
    return this.#sendRequest("autologin_timeout") as number;
  }

  /**
   * The username with which to automatically log in when the timer expires.
   */
  get autologin_user(): string {
    return this.#sendRequest("autologin_user") as string;
  }

  /**
   * Whether or not the greeter can make the system hibernate.
   */
  get can_hibernate(): boolean {
    return this.#sendRequest("can_hibernate") as boolean;
  }

  /**
   * Whether or not the greeter can make the system restart.
   */
  get can_restart(): boolean {
    return this.#sendRequest("can_restart") as boolean;
  }

  /**
   * Whether or not the greeter can make the system shutdown.
   */
  get can_shutdown(): boolean {
    return this.#sendRequest("can_shutdown") as boolean;
  }

  /**
   * Whether or not the greeter can make the system suspend/sleep.
   */
  get can_suspend(): boolean {
    return this.#sendRequest("can_suspend") as boolean;
  }

  /**
   * The name of the default session.
   */
  get default_session(): string {
    return this.#sendRequest("default_session") as string;
  }

  /**
   * Whether or not guest sessions are supported.
   */
  get has_guest_account(): boolean {
    return this.#sendRequest("has_guest_account") as boolean;
  }

  /**
   * Whether or not user accounts should be hidden.
   */
  get hide_users_hint(): boolean {
    return this.#sendRequest("hide_users_hint") as boolean;
  }

  /**
   * The system's hostname.
   */
  get hostname(): string {
    return this.#sendRequest("hostname") as string;
  }

  /**
   * Whether or not the greeter is in the process of authenticating.
   */
  get in_authentication(): boolean {
    return this.#sendRequest("in_authentication") as boolean;
  }

  /**
   * Whether or not the greeter has successfully authenticated.
   */
  get is_authenticated(): boolean {
    return this.#sendRequest("is_authenticated") as boolean;
  }

  /**
   * The current language or {@link null} if no language.
   */
  get language(): Language | null {
    return this.#sendRequest("language") as Language | null;
  }

  /**
   * A list of languages to present to the user.
   */
  get languages(): Language[] {
    return this.#sendRequest("languages") as Language[];
  }

  /**
   * The currently active layout for the selected user.
   */
  get layout(): Layout {
    return this.#sendRequest("layout") as Layout;
  }

  /**
   * Set the active layout for the selected user.
   */
  set layout(value: string) {
    this.#sendRequest("layout", [value]);
  }

  /**
   * A list of keyboard layouts to present to the user.
   */
  get layouts(): Layout[] {
    return this.#sendRequest("layouts") as Layout[];
  }

  /**
   * Whether or not the greeter was started as a lock screen.
   */
  get lock_hint(): boolean {
    return this.#sendRequest("lock_hint") as boolean;
  }

  /**
   * The available remote sessions.
   */
  get remote_sessions(): Session[] {
    return this.#sendRequest("remote_sessions") as Session[];
  }

  /**
   * Whether or not the guest account should be selected by default.
   */
  get select_guest_hint(): boolean {
    return this.#sendRequest("select_guest_hint") as boolean;
  }

  /**
   * The username to select by default.
   */
  get select_user_hint(): string {
    return this.#sendRequest("select_user_hint") as string;
  }

  /**
   * List of available sessions.
   */
  get sessions(): Session[] {
    return this.#sendRequest("sessions") as Session[];
  }

  /**
   * Check if a manual login option should be shown. If {@link true}, the theme should
   * provide a way for a username to be entered manually. Otherwise, themes that show
   * a user list may limit logins to only those users.
   */
  get show_manual_login_hint(): string {
    return this.#sendRequest("show_manual_login_hint") as string;
  }

  /**
   * Check if a remote login option should be shown. If {@link true}, the theme should provide
   * a way for a user to log into a remote desktop server.
   */
  get show_remote_login_hint(): string {
    return this.#sendRequest("show_remote_login_hint") as string;
  }

  /**
   * List of available users.
   */
  get users(): User[] {
    return this.#sendRequest("users") as User[];
  }

  get shared_data_directory() {
    return this.#sendRequest("shared_data_directory");
  }

  /**
   * Starts the authentication procedure for a user.
   * @arg {String|null} username A username or {@link null} to prompt for a username.
   */
  authenticate(username: string | null = null) {
    return this.#sendRequest("authenticate", [username]) as boolean;
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
  cancel_authentication(): boolean {
    return this.#sendRequest("cancel_authentication") as boolean;
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
  hibernate(): boolean {
    return this.#sendRequest("hibernate") as boolean;
  }

  /**
   * Provide a response to a prompt.
   */
  respond(response: string): boolean {
    return this.#sendRequest("respond", [response]) as boolean;
  }

  /**
   * Triggers the system to restart.
   * @returns {boolean} {@link true} if restart initiated, otherwise {@link false}
   */
  restart(): boolean {
    return this.#sendRequest("restart") as boolean;
  }

  /**
   * Set the language for the currently authenticated user.
   * @arg {string} language The language in the form of a locale specification (e.g. 'de_DE.UTF-8')
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  set_language(language: string): boolean {
    return this.#sendRequest("set_language", [language]) as boolean;
  }

  /**
   * Triggers the system to shutdown.
   * @returns {boolean} {@link true} if shutdown initiated, otherwise {@link false}
   */
  shutdown(): boolean {
    return this.#sendRequest("shutdown") as boolean;
  }

  /**
   * Start a session for the authenticated user.
   * @arg {String|null} session The session to log into or {@link null} to use the default.
   * @returns {boolean} {@link true} if successful, otherwise {@link false}
   */
  start_session(session: string | null): boolean {
    return this.#sendRequest("start_session", [session]) as boolean;
  }

  /**
   * Triggers the system to suspend/sleep.
   * @returns {boolean} {@link true} if suspend/sleep initiated, otherwise {@link false}
   */
  suspend(): boolean {
    return this.#sendRequest("suspend") as boolean;
  }
}

declare global {
  var lightdm: GreeterClass;
}

globalThis.greeter = new Greeter();
globalThis.lightdm = globalThis.greeter;
