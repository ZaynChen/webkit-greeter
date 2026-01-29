// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

#[cfg(feature = "greetd")]
mod greetd;
#[cfg(feature = "lightdm")]
mod lightdm;

#[cfg(feature = "greetd")]
pub use greetd::GreetdGreeter;
#[cfg(feature = "lightdm")]
pub use lightdm::LightDMGreeter;
