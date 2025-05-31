// nova_shell/src/main.rs
use nova_shell::application::launch_shell_application;
use tracing_subscriber;

fn main() {
    // Initialize tracing/logging
    // Use a default subscriber for now. Can be configured further later.
    tracing_subscriber::fmt::try_init().expect("Failed to initialize tracing subscriber");

    // Get the i32 value from glib::ExitCode
    let exit_code_value: i32 = launch_shell_application().into();
    std::process::exit(exit_code_value);
}
