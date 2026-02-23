// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

mod common;

mod greeters;
#[cfg(feature = "greetd")]
pub use greeters::GreetdGreeter;
#[cfg(feature = "lightdm")]
pub use greeters::LightDMGreeter;
