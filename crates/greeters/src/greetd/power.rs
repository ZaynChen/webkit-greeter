// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use std::sync::OnceLock;

use zbus::{blocking::Connection, proxy};

pub struct PowerManager;

impl PowerManager {
    fn proxy() -> &'static ManagerProxyBlocking<'static> {
        static MANAGER: OnceLock<ManagerProxyBlocking> = OnceLock::new();
        MANAGER.get_or_init(|| {
            let conn = Connection::system().unwrap();
            ManagerProxyBlocking::new(&conn).unwrap()
        })
    }
    /// CanHibernate method
    pub fn can_hibernate() -> bool {
        Self::proxy().can_hibernate().is_ok_and(|s| s == "yes")
    }

    /// CanPowerOff method
    pub fn can_power_off() -> bool {
        Self::proxy().can_power_off().is_ok_and(|s| s == "yes")
    }

    /// CanReboot method
    pub fn can_reboot() -> bool {
        Self::proxy().can_reboot().is_ok_and(|s| s == "yes")
    }

    /// CanSuspend method
    pub fn can_suspend() -> bool {
        Self::proxy().can_suspend().is_ok_and(|s| s == "yes")
    }

    /// Hibernate method
    pub fn hibernate() -> zbus::Result<()> {
        Self::proxy().hibernate(false)
    }

    /// PowerOff method
    pub fn power_off() -> zbus::Result<()> {
        Self::proxy().power_off(false)
    }

    /// Reboot method
    pub fn reboot() -> zbus::Result<()> {
        Self::proxy().reboot(false)
    }

    /// Suspend method
    pub fn suspend() -> zbus::Result<()> {
        Self::proxy().suspend(false)
    }
}

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait Manager {
    /// CanHibernate method
    fn can_hibernate(&self) -> zbus::Result<String>;

    /// CanPowerOff method
    fn can_power_off(&self) -> zbus::Result<String>;

    /// CanReboot method
    fn can_reboot(&self) -> zbus::Result<String>;

    /// CanSuspend method
    fn can_suspend(&self) -> zbus::Result<String>;

    /// Hibernate method
    fn hibernate(&self, interactive: bool) -> zbus::Result<()>;

    /// PowerOff method
    fn power_off(&self, interactive: bool) -> zbus::Result<()>;

    /// Reboot method
    fn reboot(&self, interactive: bool) -> zbus::Result<()>;

    /// Suspend method
    fn suspend(&self, interactive: bool) -> zbus::Result<()>;
}
