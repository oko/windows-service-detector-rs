# `windows-service-detector`

A Rust crate that provides Windows Service runtime environment detection.

See the [documentation](https://docs.rs/windows-service-detector/latest/windows_service_detector/) library documentation.

## Usage

See the provided example in `examples/service.rs` for a fully functional example.

TL;DR, in your `main.rs` you should do something like:

```rust
use windows_service_detector::is_running_as_windows_service;

fn main() {
    if is_running_as_windows_service().unwrap() {
        run_service();
    } else {
        println!("this is not a service");
    }
}
```

## Running the Example

To demonstrate the example binary running as a normal command line program:

```
cargo run --example service
```

To demonstrate the same binary running as a Windows Service, use the provided test script ***in an Administrator command prompt***:

```
.\example-service-test.ps1
```

## Development

This crate is considered feature-complete, as its sole purpose is to provide Windows Service environment detection.

If you find a bug, please report it through the GitHub issue tracker for this repository.