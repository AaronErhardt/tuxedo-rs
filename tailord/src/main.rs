use std::time::Duration;

use zbus::{dbus_interface, Connection};

struct Tailor {
    count: u64,
}

#[dbus_interface(name = "com.tux.Tailor")]
impl Tailor {
    // Can be `async` as well.
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called {} times.", name, self.count)
    }
}

fn main() {
    tokio_uring::start(async {
        start_dbus().await;
    });
}

async fn start_dbus() {
    let tailor = Tailor { count: 0 };

    let connection = Connection::system().await.unwrap();

    // setup the server
    connection
        .object_server()
        .at("/com/tux/Tailor", tailor).await.unwrap();

    connection
        .request_name("com.tux.Tailor")
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_secs(u64::MAX)).await;
}
