// Copyright 2024 Novade Co. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! D-Bus proxies for power management services.

use zbus::{zvariant::OwnedObjectPath, Proxy};

/// UPower service proxy.
#[derive(Proxy, Debug)]
#[proxy(
    interface = "org.freedesktop.UPower",
    destination = "org.freedesktop.UPower",
    path = "/org/freedesktop/UPower"
)]
pub trait UPowerProxy {
    #[proxy(method = "EnumerateDevices")]
    async fn enumerate_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;

    #[proxy(method = "GetDisplayDevice")]
    async fn get_display_device(&self) -> zbus::Result<OwnedObjectPath>;

    // Note: OnBattery is a property of the DisplayDevice, not UPower itself.
    // It might be better to fetch the display device and then get this property
    // from a UPowerDeviceProxy for that device.
    // For now, let's assume we might need a way to query it,
    // but direct implementation here might be misleading.
    // Consider if this should be on UPowerDeviceProxy for the display device.
    // #[proxy(property)]
    // async fn on_battery(&self) -> zbus::Result<bool>;
}

/// UPower device proxy.
/// The path for this proxy will be dynamic, representing a specific device.
#[derive(Proxy, Debug)]
#[proxy(
    interface = "org.freedesktop.UPower.Device",
    destination = "org.freedesktop.UPower"
)]
pub trait UPowerDeviceProxy {
    #[proxy(property)]
    async fn state(&self) -> zbus::Result<u32>;

    #[proxy(property)]
    async fn percentage(&self) -> zbus::Result<f64>;

    #[proxy(property)]
    async fn time_to_empty(&self) -> zbus::Result<i64>;

    #[proxy(property)]
    async fn time_to_full(&self) -> zbus::Result<i64>;

    #[proxy(property)]
    async fn vendor(&self) -> zbus::Result<String>;

    #[proxy(property)]
    async fn model(&self) -> zbus::Result<String>;

    #[proxy(property)]
    async fn technology(&self) -> zbus::Result<u32>;

    #[proxy(property(name = "IsPresent"))]
    async fn is_present(&self) -> zbus::Result<bool>;
    
    #[proxy(property(name = "IconName"))]
    async fn icon_name(&self) -> zbus::Result<String>;

    #[proxy(property(name = "OnBattery"))]
    async fn on_battery(&self) -> zbus::Result<bool>;

    #[proxy(signal)]
    async fn changed(&self) -> zbus::Result<()>; // TODO: This signal usually carries properties
}

/// systemd Login Manager proxy.
#[derive(Proxy, Debug)]
#[proxy(
    interface = "org.freedesktop.login1.Manager",
    destination = "org.freedesktop.login1",
    path = "/org/freedesktop/login1"
)]
pub trait LogindManagerProxy {
    #[proxy(method = "Suspend")]
    async fn suspend(&self, interactive: bool) -> zbus::Result<()>;

    #[proxy(method = "Hibernate")]
    async fn hibernate(&self, interactive: bool) -> zbus::Result<()>;

    #[proxy(method = "PowerOff")]
    async fn power_off(&self, interactive: bool) -> zbus::Result<()>;

    #[proxy(method = "Reboot")]
    async fn reboot(&self, interactive: bool) -> zbus::Result<()>;

    #[proxy(method = "CanSuspend")]
    async fn can_suspend(&self) -> zbus::Result<String>;

    #[proxy(method = "CanHibernate")]
    async fn can_hibernate(&self) -> zbus::Result<String>;

    #[proxy(method = "CanPowerOff")]
    async fn can_power_off(&self) -> zbus::Result<String>;

    #[proxy(method = "CanReboot")]
    async fn can_reboot(&self) -> zbus::Result<String>;

    // TODO: Implement Inhibit method for sleep inhibitors if time permits.
    // #[proxy(method = "Inhibit")]
    // async fn inhibit(
    //     &self,
    //     what: &str,      // e.g., "sleep", "shutdown", "idle"
    //     who: &str,       // Application name
    //     why: &str,       // Reason
    //     mode: &str,      // "block" or "delay"
    // ) -> zbus::Result<zbus::zvariant::OwnedFd>;

    // TODO: Implement IdleHint property if time permits.
    // #[proxy(property(name = "IdleHint"))]
    // async fn idle_hint(&self) -> zbus::Result<bool>;

    // TODO: Implement IdleHintChanged signal if time permits
    // #[proxy(signal(name = "IdleHintChanged"))]
    // async fn idle_hint_changed(&self, idle: bool) -> zbus::Result<()>;
}
