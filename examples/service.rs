/**
 * Simplified example service to demonstrate the use of the detector.
 *
 * When run as a binary from a command line prompt, it will print "this is not a service"
 * When run as a service (i.e. using the `example-service-test.ps1` script from the repo root),
 *     it will write out "service ran at time: $UNIXTIME" to C:\Windows\Temp\test.txt
 */
use std::error::Error;
use windows_service_detector::is_running_as_windows_service;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    if is_running_as_windows_service()? {
        service_stub::run()?;
    } else {
        println!("this is not a service");
    }
    Ok(())
}

#[cfg(windows)]
mod service_stub {
    use super::Result;
    use std::ffi::OsString;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::sync::mpsc::channel;
    use std::sync::mpsc::RecvTimeoutError;
    use std::time::Duration;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;
    use windows_service::define_windows_service;
    use windows_service::service::ServiceControl;
    use windows_service::service::ServiceControlAccept;
    use windows_service::service::ServiceExitCode;
    use windows_service::service::ServiceState;
    use windows_service::service::ServiceStatus;
    use windows_service::service::ServiceType;
    use windows_service::service_control_handler::ServiceControlHandlerResult;
    use windows_service::service_control_handler::{self};
    use windows_service::service_dispatcher;

    const SERVICE_NAME: &str = "service";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main).map_err(|e| e.into())
    }
    define_windows_service!(ffi_service_main, service_main);

    pub fn service_main(_args: Vec<OsString>) {
        if let Err(_e) = run_service() {}
    }

    pub fn run_service() -> Result<()> {
        let (shutdown_tx, shutdown_rx) = channel();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                ServiceControl::Stop => {
                    shutdown_tx.send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        let mut fh = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open("C:/Windows/Temp/test.txt")?;
        write!(
            fh,
            "service ran at time: {}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        )?;

        loop {
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) | Err(RecvTimeoutError::Disconnected) => break,
                Err(RecvTimeoutError::Timeout) => (),
            };
        }

        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }
}
