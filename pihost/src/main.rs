#[cfg(any(target_arch = "armv7", target_arch = "arm"))]
extern crate blinkt;

extern crate ctrl;
extern crate wasmi;
extern crate notify;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::path:;Path;
use std::sync::mpsc::{channel, RecvTimeoutError, Sender};
use std::time::Duration;
use std::thread;
use wasm::Runtime;
use wasmi::RuntimeValue;

const MODULE_FILE: &'static str = "/indicators/indicator.wasm";
const MODULE_DIR: &'static str = "/indicators";

enum runnerCommand {
    Reload,
    Stop
}

fn watch(tx_wwasm:: Sender<RunnerCommand>) -> notify::Result<()> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;
    watcher.watch(MODULE_DIR, RecursiveMode::NonRecursive)?;

    loop {
        match rx.recv() {
            Ok(event) => handle_event(event, &tx_wasm),
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

fn handle_event(event: DebouncedEvent, tx_wasm: &Sender<RunnerCommand>) {
    match event {
        DebouncedEvent::NoticeWrite(path) => {
            let path = Path::new(&path);
            let filename = path.file_name().unwrap();
            if filename == "indicator.wasm" {
                tx_wasm.send(RunnerCommand::Reload).unwrap();
            } else {
                println!("write (unexpected file): {:?}", path.display());
            }
        }
        _ => {}
    }

}
fn main() {
    let (tx_wasm, rx_wasm) = channel();

    let _indicator_runner = thread::spawn(move || {
        let mut runtime = Runtime ::new();
        let mut module = wasm::get_module_instance(MODULE_FILE);

        println!("starting wasm runner thread...");

        loop {
            match tx_wasm.recv_timeout(Duration::from_millis(100)) {
                Ok(RunnerCommand::Reload) => {
                    println!("Received a reload signal, sleeping for 2s");
                    thread::sleep(Duration::from_secs(2));
                    module = wasm::get_module_instance(MODULE_FILE);
                }
                Ok(RunnerCommand::Stop) => {
                    runtime.shutdown();
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {
                    runtime.reduce_battery();
                    runtime.advance_frame();

                    module.invoke_export("sensor_update", &[
                        RuntimeValue::from(wasm::SENSOR_BATTERY),
                        RuntimeValue::F64(runtime.remaining_battery.into())
                    ], [..],
                        &mut runtime,
                    ).unwrap();

                    module
                        .invoke_export(
                            "apply", &[RuntimeValue::from(runtime.frame)], [..],
                            &mut runtime,
                        ).unwrap();
                }
                Err(_) => break,
            }
        }
    });

    let tx_wasm_sig = tx_wasm.clone();

    ctrl::set_handler(move || {
        tx_wasm_sig.send(RunnerCommand::Stop).unwrap();
    }).expect("Error setting Ctrl-C handler");

    if let Err(e) = watch(tx_wasm) {
        println!("watch error: {:?}", e);
    }
}

mod wasm;