use futures_lite::StreamExt;
use once_cell::sync::Lazy;
use tokio::sync::broadcast;
use zbus::{dbus_proxy, Connection};

static SUSPEND_CHANNEL: Lazy<(broadcast::Sender<bool>, broadcast::Receiver<bool>)> =
    Lazy::new(|| broadcast::channel(1));

pub fn get_suspend_receiver() -> broadcast::Receiver<bool> {
    SUSPEND_CHANNEL.0.subscribe()
}

#[dbus_proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Suspend {
    #[dbus_proxy(signal)]
    fn prepare_for_sleep(&self, arg1: bool) -> fdo::Result<()>;
}

pub async fn wait_for_suspend() -> Result<(), zbus::Error> {
    let sender = SUSPEND_CHANNEL.0.clone();
    let connection = Connection::system().await?;
    let proxy = SuspendProxy::new(&connection).await?;
    let mut receiver = proxy.receive_prepare_for_sleep().await?;

    while let Some(msg) = receiver.next().await {
        let value = *msg.args()?.arg1();

        if value {
            tracing::info!("Suspended, sleeping until wake up.");
        } else {
            tracing::info!("Woken up, continue service.");
        };

        if let Err(err) = sender.send(value) {
            tracing::warn!("Error sending shutdown signal: `{err}`");
        }
    }

    Ok(())
}

pub async fn process_suspend(receiver: &mut broadcast::Receiver<bool>) {
    match receiver.recv().await {
        Ok(msg) => {
            // Suspended!
            if msg {
                wait_for_wake_up(receiver).await
            } else {
                tracing::warn!("Wake up message without suspend.");
            }
        }
        Err(err) => {
            tracing::error!("Filed receiving suspend message: `{err}`");
        }
    }
}

async fn wait_for_wake_up(receiver: &mut broadcast::Receiver<bool>) {
    // Wait until wake up (suspend msg == false).
    loop {
        match receiver.recv().await {
            Ok(msg) => {
                if msg {
                    tracing::warn!("Wake up message without suspend.");
                } else {
                    return;
                }
            }
            Err(err) => {
                tracing::error!("Filed receiving suspend message: `{err}`");
            }
        }
    }
}
