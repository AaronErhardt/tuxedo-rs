use futures_lite::StreamExt;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use zbus::{dbus_proxy, fdo, zvariant::OwnedObjectPath, Connection};

use std::{
    borrow::Borrow,
    future::pending,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::util;

static POWER_CONNECTED: AtomicBool = AtomicBool::new(true);

pub fn is_power_connected() -> bool {
    POWER_CONNECTED.load(Ordering::SeqCst)
}

fn set_power_connected(value: bool) {
    POWER_CONNECTED.store(value, Ordering::SeqCst)
}

static POWER_SUPPLY_CHANNEL: Lazy<(broadcast::Sender<bool>, broadcast::Receiver<bool>)> =
    Lazy::new(|| broadcast::channel(1));

pub fn get_power_supply_receiver() -> broadcast::Receiver<bool> {
    POWER_SUPPLY_CHANNEL.0.subscribe()
}

#[dbus_proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
trait UPower {
    fn enumerate_devices(&self) -> fdo::Result<Vec<OwnedObjectPath>>;
}

#[dbus_proxy(
    interface = "org.freedesktop.UPower.Device",
    default_service = "org.freedesktop.UPower"
)]
trait UPowerDevice {
    #[dbus_proxy(property)]
    fn online(&self) -> fdo::Result<bool>;

    #[dbus_proxy(property)]
    fn capacity(&self) -> fdo::Result<f64>;
}

pub async fn wait_for_power_supply_changes() {
    if let Some(connection) = util::system_bus_connection() {
        let mut sender = POWER_SUPPLY_CHANNEL.0.clone();

        // Don't try to reconnect anymore after 3 attempts
        for _ in 0..3 {
            tracing::info!("Setting up power supply watcher service");
            if let Err(err) = try_wait_for_power_supply_changes(connection, &mut sender).await {
                tracing::error!("Failed to watch for power supply events: `{err}`");
                // Reconnect after 10s
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
        }
        tracing::warn!("Stopping power supply watcher service after 3 errors");
    } else {
        tracing::warn!(
            "Stopping power supply watcher service due to missing system DBUS connection"
        );
    }
}

async fn try_wait_for_power_supply_changes(
    connection: &zbus::Connection,
    sender: &mut broadcast::Sender<bool>,
) -> Result<(), zbus::Error> {
    let proxy = get_ac_device(connection).await?;
    let mut receiver = proxy.receive_online_changed().await;

    while let Some(msg) = receiver.next().await {
        let value = msg.get().await?;

        if value {
            tracing::info!("Power supply plugged in");
        } else {
            tracing::info!("Power supply unplugged");
        };

        if let Err(err) = sender.send(value) {
            tracing::warn!("Error sending power supply event: `{err}`");
        }
    }

    Ok(())
}

async fn get_ac_device(connection: &Connection) -> fdo::Result<UPowerDeviceProxy> {
    let power_proxy = UPowerProxy::builder(&connection).build().await?;
    let devices = power_proxy.enumerate_devices().await?;

    for device in devices {
        let device_proxy = UPowerDeviceProxy::builder(&connection)
            .path(device)?
            .build()
            .await?;
        let capacity = device_proxy.capacity().await?;

        // Check whether it is a power supply (capacity should be basically 0)
        if capacity < 1e-10 {
            return Ok(device_proxy);
        }
    }
    Err(fdo::Error::NotSupported("No power supply found".to_owned()))
}

pub async fn battery_supported(connection: &Connection) -> fdo::Result<bool> {
    let power_proxy = UPowerProxy::builder(&connection).build().await?;
    let devices = power_proxy.enumerate_devices().await?;

    // At least power supply and battery are necessary.
    if devices.len() < 2 {
        return Ok(false);
    }

    let mut battery_available = false;
    let mut power_supply_available = false;
    for device in devices {
        let device_proxy = UPowerDeviceProxy::builder(&connection)
            .path(device)?
            .build()
            .await?;
        let capacity = device_proxy.capacity().await?;

        // Check whether it is a power supply (capacity should be basically 0)
        if capacity < 1e-10 {
            power_supply_available = true;
        } else {
            battery_available = true;
        }
    }
    Ok(battery_available && power_supply_available)
}

pub async fn power_supply_active() -> bool {
    if let Some(connection) = util::system_bus_connection() {
        if let Ok(device) = get_ac_device(connection).await {
            device.online().await.unwrap_or(true)
        } else {
            // If no power supply device could be found we assume that
            // the device has no battery.
            true
        }
    } else {
        true
    }
}

#[cfg(test)]
mod test {
    use super::get_ac_device;

    #[tokio::test]
    async fn test_ac_dbus_interface_connection() {
        let connection = zbus::Connection::system().await.unwrap();
        get_ac_device(&connection).await.unwrap();
    }
}
