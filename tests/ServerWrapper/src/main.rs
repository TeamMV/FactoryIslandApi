use api::server::{startup_internal_server, ServerSync};

fn main() {
    startup_internal_server(true, ServerSync::no_sync());
}
