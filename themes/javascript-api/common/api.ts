// SPDX-FileCopyrightText: 2026 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: MIT

import type { GreeterRequestMethod, GreeterRequestTarget } from "../types.d.ts";

export function sendRequest<T extends GreeterRequestTarget, Arg>(
  target: T,
  method: GreeterRequestMethod[T],
  args: Arg[] = [],
) {
  return globalThis.send_request({
    target,
    method,
    args: JSON.stringify(args),
  });
}
