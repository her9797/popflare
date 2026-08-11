mod effect;
mod platform;

use platform::PlatformApp;

fn main() {
    let mut app = PlatformApp::new();
    app.run();
}
