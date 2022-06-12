use futures_lite::StreamExt;
use tokio::sync::broadcast;
use tracing::info;
use zbus::{dbus_proxy, Connection};

#[dbus_proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Suspend {
    #[dbus_proxy(signal)]
    fn prepare_for_sleep(&self, arg1: bool) -> fdo::Result<()>;
}

pub async fn wait_for_suspend(sender: broadcast::Sender<bool>) -> Result<(), zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = SuspendProxy::new(&connection).await?;
    let mut receiver = proxy.receive_prepare_for_sleep().await?;

    while let Some(msg) = receiver.next().await {
        let value = *msg.args()?.arg1();

        if value {
            info!("Suspended, sleeping until wake up.");
        } else {
            info!("Woken up, continue service.");
        };

        let _ = sender.send(value);
    }

    Ok(())
}
