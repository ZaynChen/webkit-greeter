// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

type Language = {
  code: string;
  name: string;
  territory: string;
};

type Layout = {
  name: string;
  description: string;
  short_description: string | null;
};

type Session = {
  key: string;
  name: string;
  type: string;
  comment: string;
};

type User = {
  display_name: string;
  home_directory: string;
  image: string;
  language: string;
  session: string;
  username: string;
  // lightdm only
  background?: string | null;
  logged_in?: boolean;
  layout?: string | null;
  layouts?: string[] | null;
};

type GreeterConfigBranding = {
  background_images_dir: string;
  logo_image: string;
  user_image: string;
};

type GreeterConfigGreeter = {
  debug_mode: boolean;
  detect_theme_errors: boolean;
  screensaver_timeout: number;
  secure_mode: boolean;
  theme: string;
  icon_theme: string | null;
  time_language: string | null;
};

type GreeterRequestTarget =
  | "greeter"
  | "greeter_config"
  | "greeter_comm"
  | "theme_utils";

type GreeterRequestMethod = {
  greeter:
    // common
    | "can_hibernate"
    | "can_restart"
    | "can_shutdown"
    | "can_suspend"
    | "hibernate"
    | "restart"
    | "shutdown"
    | "suspend"
    | "language"
    | "languages"
    | "layout"
    | "layouts"
    | "sessions"
    | "users"
    // greetd + lightdm
    | "authentication_user"
    | "in_authentication"
    | "is_authenticated"
    | "cancel_authentication"
    | "authenticate"
    | "respond"
    | "start_session"
    // only lightdm
    | "autologin_guest"
    | "autologin_timeout"
    | "autologin_user"
    | "default_session"
    | "has_guest_account"
    | "hide_users_hint"
    | "hostname"
    | "lock_hint"
    | "remote_sessions"
    | "select_guest_hint"
    | "select_user_hint"
    | "show_manual_login_hint"
    | "show_remote_login_hint"
    | "shared_data_directory"
    | "set_language"
    | "authenticate_as_guest"
    | "cancel_autologin";
  greeter_comm: "broadcast";
  greeter_config: "branding" | "greeter";
  theme_utils: "dirlist";
};

type GreeterRequestReturnType =
  | null
  | undefined
  | boolean
  | string
  | string[]
  | Language
  | Language[]
  | Layout
  | Layout[]
  | Session[]
  | User[]
  | GreeterConfigBranding
  | GreeterConfigGreeter
  | number;

declare global {
  function send_request<T extends GreeterRequestTarget>(
    request: { target: T; method: GreeterRequestMethod[T]; args: string },
  ): GreeterRequestReturnType;
  var greeter: Greeter;
  var greeter_comm: GreeterComm;
  var greeter_config: GreeterConfig;
  var theme_utils: ThemeUtils;
  function dispatch_ready_event(): boolean;
}

export type {
  GreeterConfigBranding,
  GreeterConfigGreeter,
  GreeterRequestMethod,
  GreeterRequestTarget,
  Language,
  Layout,
  Session,
  User,
};

export class GreeterComm {
  broadcast<T>(data: T): void;
  _emit<T>(data: T): boolean;
}

export class GreeterConfig {
  get branding(): GreeterConfigBranding;
  get greeter(): GreeterConfigGreeter;
  get layouts(): Layout[];
}

export class ThemeUtils {
  dirlist(path: string, only_image?: boolean): Promise<string[]>;
}

export class Signal {
  connect(callback: (...args: string[]) => void): void;
  disconnect(callback: (...args: string[]) => void): void;
  _emit(...args: string[]): void;
}

export class Greeter {
  show_prompt: Signal;
  show_message: Signal;
  authentication_complete: Signal;
  get can_hibernate(): boolean;
  get can_restart(): boolean;
  get can_shutdown(): boolean;
  get can_suspend(): boolean;
  hibernate(): boolean;
  restart(): boolean;
  shutdown(): boolean;
  suspend(): boolean;
  get language(): Language | null;
  get languages(): Language[];
  get layout(): Layout;
  set layout(layout: string);
  get layouts(): Layout[];
  get sessions(): Session[];
  get users(): User[];
  get authentication_user(): string | null;
  get in_authentication(): boolean;
  get is_authenticated(): boolean;
  authenticate(username: string | null): boolean;
  cancel_authentication(): boolean;
  respond(password?: string | null): boolean;
  start_session(session: string): boolean;
}
