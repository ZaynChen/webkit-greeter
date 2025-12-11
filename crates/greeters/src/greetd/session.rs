// SPDX-FileCopyrightText: 2025 ZaynChen <zaynchen@qq.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later AND LGPL-3.0-or-later

use webkit::glib::{
    KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_HIDDEN, KEY_FILE_DESKTOP_KEY_NO_DISPLAY,
    KEY_FILE_DESKTOP_KEY_TRY_EXEC, KeyFile, KeyFileFlags, find_program_in_path, system_data_dirs,
};
use zbus::blocking::Connection;

use std::{collections::HashMap, fs::read_dir, path::PathBuf, sync::OnceLock};

#[derive(Debug)]
pub struct Session {
    key: String,
    type_: String,
    name: String,
    comment: String,
    exec: String,
}

impl Session {
    fn new(key: String, type_: String, name: String, comment: String, exec: String) -> Self {
        Self {
            key,
            type_,
            name,
            comment,
            exec,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn type_(&self) -> &str {
        &self.type_
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn comment(&self) -> &str {
        &self.comment
    }

    pub fn exec(&self) -> &str {
        &self.exec
    }
}

pub struct SessionManager;
impl SessionManager {
    fn manager() -> &'static ManagerProxyBlocking<'static> {
        static MANAGER: OnceLock<ManagerProxyBlocking> = OnceLock::new();
        MANAGER.get_or_init(|| ManagerProxyBlocking::new(&Connection::system().unwrap()).unwrap())
    }

    // TODO: handle duplicated sessions
    fn available_sessions_map() -> &'static HashMap<String, Session> {
        static SESSION_FILES: OnceLock<HashMap<String, Session>> = OnceLock::new();
        SESSION_FILES.get_or_init(|| {
            system_data_dirs() // ["/usr/local/share", "/usr/share"]
                .iter()
                .flat_map(|dir| {
                    [
                        load_session_dir(dir.join("xsessions"), "x"),
                        load_session_dir(dir.join("wayland-sessions"), "wayland"),
                    ]
                    .into_iter()
                    .flatten()
                    .collect::<HashMap<_, _>>()
                })
                .collect()
        })
    }

    pub fn is_logged_in(uid: u32) -> bool {
        Self::manager().get_user(uid).is_ok()
    }

    pub fn sessions() -> Vec<&'static Session> {
        let mut sessions: Vec<_> = Self::available_sessions_map().values().collect();
        sessions.sort_by_key(|s| s.key());
        sessions
    }

    pub fn session(key: &str) -> Option<&Session> {
        Self::available_sessions_map().get(key)
    }
}

fn is_session_desktop_file(keyfile: &KeyFile) -> bool {
    let no_display = keyfile
        .boolean(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_NO_DISPLAY)
        .is_ok_and(|no_display| no_display);
    let hidden = keyfile
        .boolean(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_HIDDEN)
        .is_ok_and(|hidden| hidden);
    let tryexec_failed = keyfile
        .string(KEY_FILE_DESKTOP_GROUP, KEY_FILE_DESKTOP_KEY_TRY_EXEC)
        .is_ok_and(|try_exec| find_program_in_path(try_exec).is_none());
    !no_display && !hidden && !tryexec_failed
}

fn load_session_dir(dir: PathBuf, session_type: &str) -> HashMap<String, Session> {
    if !dir.is_dir() {
        return HashMap::with_capacity(0);
    }
    read_dir(dir)
        .unwrap()
        .filter_map(|ent| ent.ok())
        .filter_map(|file| {
            let keyfile = KeyFile::new();
            let filepath = file.path();
            let filepath_str = filepath.to_str().unwrap();
            if let Err(e) = keyfile.load_from_file(&filepath, KeyFileFlags::NONE) {
                logger::warn!("Failed to load \"{filepath_str}\": {e}");
            } else if keyfile.has_group(KEY_FILE_DESKTOP_GROUP) {
                if !is_session_desktop_file(&keyfile) {
                    logger::warn!(
                        "\"{filepath_str}\" is hidden, {}, {}",
                        "contains non-executable TryExec program",
                        "or is otherwise not capable of being used"
                    );
                } else if keyfile
                    .has_key(KEY_FILE_DESKTOP_GROUP, "Name")
                    .is_ok_and(|b| b)
                    && keyfile
                        .has_key(KEY_FILE_DESKTOP_GROUP, "Exec")
                        .is_ok_and(|b| b)
                {
                    let key = file
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".desktop")
                        .to_string();
                    let name = keyfile
                        .locale_string(KEY_FILE_DESKTOP_GROUP, "Name", None)
                        .unwrap()
                        .into();
                    let comment = keyfile
                        .locale_string(KEY_FILE_DESKTOP_GROUP, "Comment", None)
                        .unwrap_or_default()
                        .into();
                    let exec = keyfile
                        .string(KEY_FILE_DESKTOP_GROUP, "Exec")
                        .unwrap_or_default()
                        .into();
                    return Some((
                        key.clone(),
                        Session::new(key, session_type.into(), name, comment, exec),
                    ));
                } else {
                    logger::warn!("{filepath_str} contains no \"Name\" or \"Exec\" key");
                }
            }
            None
        })
        .collect()
}

// TODO: remove unused api
use zbus::proxy;
#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
pub trait Manager {
    /// ActivateSession method
    fn activate_session(&self, session_id: &str) -> zbus::Result<()>;

    /// ActivateSessionOnSeat method
    fn activate_session_on_seat(&self, session_id: &str, seat_id: &str) -> zbus::Result<()>;

    /// GetSession method
    fn get_session(&self, session_id: &str) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// GetSessionByPID method
    #[zbus(name = "GetSessionByPID")]
    fn get_session_by_pid(&self, pid: u32) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// GetUser method
    fn get_user(&self, uid: u32) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// GetUserByPID method
    #[zbus(name = "GetUserByPID")]
    fn get_user_by_pid(&self, pid: u32) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Halt method
    fn halt(&self, interactive: bool) -> zbus::Result<()>;

    /// HaltWithFlags method
    fn halt_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// HibernateWithFlags method
    fn hibernate_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// HybridSleep method
    fn hybrid_sleep(&self, interactive: bool) -> zbus::Result<()>;

    /// HybridSleepWithFlags method
    fn hybrid_sleep_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// Inhibit method
    fn inhibit(
        &self,
        what: &str,
        who: &str,
        why: &str,
        mode: &str,
    ) -> zbus::Result<zbus::zvariant::OwnedFd>;

    /// KillSession method
    fn kill_session(&self, session_id: &str, whom: &str, signal_number: i32) -> zbus::Result<()>;

    /// KillUser method
    fn kill_user(&self, uid: u32, signal_number: i32) -> zbus::Result<()>;

    /// ListSeats method
    fn list_seats(&self) -> zbus::Result<Vec<(String, zbus::zvariant::OwnedObjectPath)>>;

    /// ListUsers method
    fn list_users(&self) -> zbus::Result<Vec<(u32, String, zbus::zvariant::OwnedObjectPath)>>;

    /// LockSession method
    fn lock_session(&self, session_id: &str) -> zbus::Result<()>;

    /// LockSessions method
    fn lock_sessions(&self) -> zbus::Result<()>;

    /// PowerOffWithFlags method
    fn power_off_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// RebootWithFlags method
    fn reboot_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// ReleaseSession method
    fn release_session(&self, session_id: &str) -> zbus::Result<()>;

    /// ScheduleShutdown method
    fn schedule_shutdown(&self, type_: &str, usec: u64) -> zbus::Result<()>;

    /// SetRebootParameter method
    fn set_reboot_parameter(&self, parameter: &str) -> zbus::Result<()>;

    /// SetRebootToBootLoaderEntry method
    fn set_reboot_to_boot_loader_entry(&self, boot_loader_entry: &str) -> zbus::Result<()>;

    /// SetRebootToBootLoaderMenu method
    fn set_reboot_to_boot_loader_menu(&self, timeout: u64) -> zbus::Result<()>;

    /// SetRebootToFirmwareSetup method
    fn set_reboot_to_firmware_setup(&self, enable: bool) -> zbus::Result<()>;

    /// SetUserLinger method
    fn set_user_linger(&self, uid: u32, enable: bool, interactive: bool) -> zbus::Result<()>;

    /// SetWallMessage method
    fn set_wall_message_with_enable(&self, wall_message: &str, enable: bool) -> zbus::Result<()>;

    /// Sleep method
    fn sleep(&self, flags: u64) -> zbus::Result<()>;

    /// SuspendThenHibernate method
    fn suspend_then_hibernate(&self, interactive: bool) -> zbus::Result<()>;

    /// SuspendThenHibernateWithFlags method
    fn suspend_then_hibernate_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// SuspendWithFlags method
    fn suspend_with_flags(&self, flags: u64) -> zbus::Result<()>;

    /// TerminateSeat method
    fn terminate_seat(&self, seat_id: &str) -> zbus::Result<()>;

    /// TerminateSession method
    fn terminate_session(&self, session_id: &str) -> zbus::Result<()>;

    /// TerminateUser method
    fn terminate_user(&self, uid: u32) -> zbus::Result<()>;

    /// UnlockSession method
    fn unlock_session(&self, session_id: &str) -> zbus::Result<()>;

    /// UnlockSessions method
    fn unlock_sessions(&self) -> zbus::Result<()>;

    /// PrepareForShutdown signal
    #[zbus(signal)]
    fn prepare_for_shutdown(&self, start: bool) -> zbus::Result<()>;

    /// PrepareForShutdownWithMetadata signal
    #[zbus(signal)]
    fn prepare_for_shutdown_with_metadata(
        &self,
        start: bool,
        metadata: std::collections::HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<()>;

    /// PrepareForSleep signal
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;

    /// SeatNew signal
    #[zbus(signal)]
    fn seat_new(
        &self,
        seat_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// SeatRemoved signal
    #[zbus(signal)]
    fn seat_removed(
        &self,
        seat_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// SecureAttentionKey signal
    #[zbus(signal)]
    fn secure_attention_key(
        &self,
        seat_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// SessionNew signal
    #[zbus(signal)]
    fn session_new(
        &self,
        session_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// SessionRemoved signal
    #[zbus(signal)]
    fn session_removed(
        &self,
        session_id: &str,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// UserNew signal
    #[zbus(signal)]
    fn user_new(&self, uid: u32, object_path: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// UserRemoved signal
    #[zbus(signal)]
    fn user_removed(
        &self,
        uid: u32,
        object_path: zbus::zvariant::ObjectPath<'_>,
    ) -> zbus::Result<()>;

    /// BlockInhibited property
    #[zbus(property)]
    fn block_inhibited(&self) -> zbus::Result<String>;

    /// BlockWeakInhibited property
    #[zbus(property)]
    fn block_weak_inhibited(&self) -> zbus::Result<String>;

    /// BootLoaderEntries property
    #[zbus(property)]
    fn boot_loader_entries(&self) -> zbus::Result<Vec<String>>;

    /// DelayInhibited property
    #[zbus(property)]
    fn delay_inhibited(&self) -> zbus::Result<String>;

    /// DesignatedMaintenanceTime property
    #[zbus(property)]
    fn designated_maintenance_time(&self) -> zbus::Result<String>;

    /// Docked property
    #[zbus(property)]
    fn docked(&self) -> zbus::Result<bool>;

    /// EnableWallMessages property
    #[zbus(property)]
    fn enable_wall_messages(&self) -> zbus::Result<bool>;
    #[zbus(property)]
    fn set_enable_wall_messages(&self, value: bool) -> zbus::Result<()>;

    /// HandleHibernateKey property
    #[zbus(property)]
    fn handle_hibernate_key(&self) -> zbus::Result<String>;

    /// HandleHibernateKeyLongPress property
    #[zbus(property)]
    fn handle_hibernate_key_long_press(&self) -> zbus::Result<String>;

    /// HandleLidSwitch property
    #[zbus(property)]
    fn handle_lid_switch(&self) -> zbus::Result<String>;

    /// HandleLidSwitchDocked property
    #[zbus(property)]
    fn handle_lid_switch_docked(&self) -> zbus::Result<String>;

    /// HandleLidSwitchExternalPower property
    #[zbus(property)]
    fn handle_lid_switch_external_power(&self) -> zbus::Result<String>;

    /// HandlePowerKey property
    #[zbus(property)]
    fn handle_power_key(&self) -> zbus::Result<String>;

    /// HandlePowerKeyLongPress property
    #[zbus(property)]
    fn handle_power_key_long_press(&self) -> zbus::Result<String>;

    /// HandleRebootKey property
    #[zbus(property)]
    fn handle_reboot_key(&self) -> zbus::Result<String>;

    /// HandleRebootKeyLongPress property
    #[zbus(property)]
    fn handle_reboot_key_long_press(&self) -> zbus::Result<String>;

    /// HandleSecureAttentionKey property
    #[zbus(property)]
    fn handle_secure_attention_key(&self) -> zbus::Result<String>;

    /// HandleSuspendKey property
    #[zbus(property)]
    fn handle_suspend_key(&self) -> zbus::Result<String>;

    /// HandleSuspendKeyLongPress property
    #[zbus(property)]
    fn handle_suspend_key_long_press(&self) -> zbus::Result<String>;

    /// HoldoffTimeoutUSec property
    #[zbus(property, name = "HoldoffTimeoutUSec")]
    fn holdoff_timeout_usec(&self) -> zbus::Result<u64>;

    /// IdleAction property
    #[zbus(property)]
    fn idle_action(&self) -> zbus::Result<String>;

    /// IdleActionUSec property
    #[zbus(property, name = "IdleActionUSec")]
    fn idle_action_usec(&self) -> zbus::Result<u64>;

    /// IdleHint property
    #[zbus(property)]
    fn idle_hint(&self) -> zbus::Result<bool>;

    /// IdleSinceHint property
    #[zbus(property)]
    fn idle_since_hint(&self) -> zbus::Result<u64>;

    /// IdleSinceHintMonotonic property
    #[zbus(property)]
    fn idle_since_hint_monotonic(&self) -> zbus::Result<u64>;

    /// InhibitDelayMaxUSec property
    #[zbus(property, name = "InhibitDelayMaxUSec")]
    fn inhibit_delay_max_usec(&self) -> zbus::Result<u64>;

    /// InhibitorsMax property
    #[zbus(property)]
    fn inhibitors_max(&self) -> zbus::Result<u64>;

    /// KillExcludeUsers property
    #[zbus(property)]
    fn kill_exclude_users(&self) -> zbus::Result<Vec<String>>;

    /// KillOnlyUsers property
    #[zbus(property)]
    fn kill_only_users(&self) -> zbus::Result<Vec<String>>;

    /// KillUserProcesses property
    #[zbus(property)]
    fn kill_user_processes(&self) -> zbus::Result<bool>;

    /// LidClosed property
    #[zbus(property)]
    fn lid_closed(&self) -> zbus::Result<bool>;

    /// NAutoVTs property
    #[zbus(property, name = "NAutoVTs")]
    fn nauto_vts(&self) -> zbus::Result<u32>;

    /// NCurrentInhibitors property
    #[zbus(property, name = "NCurrentInhibitors")]
    fn ncurrent_inhibitors(&self) -> zbus::Result<u64>;

    /// NCurrentSessions property
    #[zbus(property, name = "NCurrentSessions")]
    fn ncurrent_sessions(&self) -> zbus::Result<u64>;

    /// OnExternalPower property
    #[zbus(property)]
    fn on_external_power(&self) -> zbus::Result<bool>;

    /// PreparingForShutdown property
    #[zbus(property)]
    fn preparing_for_shutdown(&self) -> zbus::Result<bool>;

    /// PreparingForShutdownWithMetadata property
    #[zbus(property)]
    fn preparing_for_shutdown_with_metadata(
        &self,
    ) -> zbus::Result<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>;

    /// PreparingForSleep property
    #[zbus(property)]
    fn preparing_for_sleep(&self) -> zbus::Result<bool>;

    /// RebootParameter property
    #[zbus(property)]
    fn reboot_parameter(&self) -> zbus::Result<String>;

    /// RebootToBootLoaderEntry property
    #[zbus(property)]
    fn reboot_to_boot_loader_entry(&self) -> zbus::Result<String>;

    /// RebootToBootLoaderMenu property
    #[zbus(property)]
    fn reboot_to_boot_loader_menu(&self) -> zbus::Result<u64>;

    /// RebootToFirmwareSetup property
    #[zbus(property)]
    fn reboot_to_firmware_setup(&self) -> zbus::Result<bool>;

    /// RemoveIPC property
    #[zbus(property, name = "RemoveIPC")]
    fn remove_ipc(&self) -> zbus::Result<bool>;

    /// RuntimeDirectoryInodesMax property
    #[zbus(property)]
    fn runtime_directory_inodes_max(&self) -> zbus::Result<u64>;

    /// RuntimeDirectorySize property
    #[zbus(property)]
    fn runtime_directory_size(&self) -> zbus::Result<u64>;

    /// ScheduledShutdown property
    #[zbus(property)]
    fn scheduled_shutdown(&self) -> zbus::Result<(String, u64)>;

    /// SessionsMax property
    #[zbus(property)]
    fn sessions_max(&self) -> zbus::Result<u64>;

    /// SleepOperation property
    #[zbus(property)]
    fn sleep_operation(&self) -> zbus::Result<Vec<String>>;

    /// StopIdleSessionUSec property
    #[zbus(property, name = "StopIdleSessionUSec")]
    fn stop_idle_session_usec(&self) -> zbus::Result<u64>;

    /// UserStopDelayUSec property
    #[zbus(property, name = "UserStopDelayUSec")]
    fn user_stop_delay_usec(&self) -> zbus::Result<u64>;

    /// WallMessage property
    #[zbus(property)]
    fn wall_message(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_wall_message(&self, value: &str) -> zbus::Result<()>;
}
