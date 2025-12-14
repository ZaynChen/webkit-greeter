// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use super::dbus::LogindManager;

pub struct PowerManager;

impl PowerManager {
    /// CanHibernate method
    pub fn can_hibernate() -> bool {
        LogindManager::proxy()
            .can_hibernate()
            .is_ok_and(|s| s == "yes")
    }

    /// CanPowerOff method
    pub fn can_power_off() -> bool {
        LogindManager::proxy()
            .can_power_off()
            .is_ok_and(|s| s == "yes")
    }

    /// CanReboot method
    pub fn can_reboot() -> bool {
        LogindManager::proxy()
            .can_reboot()
            .is_ok_and(|s| s == "yes")
    }

    /// CanSuspend method
    pub fn can_suspend() -> bool {
        LogindManager::proxy()
            .can_suspend()
            .is_ok_and(|s| s == "yes")
    }

    /// Hibernate method
    pub fn hibernate() -> zbus::Result<()> {
        LogindManager::proxy().hibernate(false)
    }

    /// PowerOff method
    pub fn power_off() -> zbus::Result<()> {
        LogindManager::proxy().power_off(false)
    }

    /// Reboot method
    pub fn reboot() -> zbus::Result<()> {
        LogindManager::proxy().reboot(false)
    }

    /// Suspend method
    pub fn suspend() -> zbus::Result<()> {
        LogindManager::proxy().suspend(false)
    }
}
