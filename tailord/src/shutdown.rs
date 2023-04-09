use futures::StreamExt;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::sync::broadcast;

pub fn setup() -> broadcast::Receiver<()> {
    // Setup shutdown
    let (shutdown_sender, shutdown_receiver) = broadcast::channel(1);

    let signals = Signals::new([SIGTERM, SIGINT, SIGQUIT]).unwrap();
    tracing::debug!("Starting signal handler runtime");
    tokio_uring::spawn(handle_signals(signals, shutdown_sender));

    shutdown_receiver
}

#[tracing::instrument(skip(signals))]
async fn handle_signals(mut signals: Signals, shutdown_sender: broadcast::Sender<()>) {
    tracing::debug!("Waiting for signals");
    while let Some(signal) = signals.next().await {
        tracing::debug!("Caught signal {signal}");
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                // It's ok to panic here if a send error occurs.
                // The application is terminated anyway and
                // an error at this point can't be recovered.
                tracing::info!("Received a shutdown signal");
                shutdown_sender.send(()).unwrap();
            }
            _ => unreachable!(),
        }
    }
}
