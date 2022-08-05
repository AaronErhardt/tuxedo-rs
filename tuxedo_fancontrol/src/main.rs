use std::io::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::fan::fan_control_loop;
use crate::config::Instance;

mod config;
mod fan;

fn main() -> Result<(), Error> {
    println!("tuxedo_fancontrol v0.1, starting.");
    
    // get root privileges
    sudo::escalate_if_needed().unwrap();
   
    // prepare to manage I/O
    // loading global variables and config
    println!("Loading configuration file...");
    let mut i = Instance::init();


    // basic configuration consistency checks
    i.config.check();
    
    // handle interrupts
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&term))?;
  
    // loop until it recieves interrupt
    while !term.load(Ordering::Relaxed) {
        fan_control_loop(&mut i);
    }
    // reset fan speed to auto
    i.io.set_fans_auto().unwrap();

    // exiting the loop and shutting down
    println!("Interrupt received, exiting gracefully.");
    Ok(())
}

