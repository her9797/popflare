#![allow(deprecated, unexpected_cfgs, unsafe_op_in_unsafe_fn)]

use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::{Mutex, Once, OnceLock};

use block::ConcreteBlock;
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicyAccessory, NSBackingStoreBuffered, NSColor,
    NSImage, NSMenu, NSMenuItem, NSScreen, NSStatusBar, NSVariableStatusItemLength, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::effect::{FlareEngine, Point};

const LEFT_MOUSE_DOWN_MASK: u64 = 1 << 1;
const FLOATING_WINDOW_LEVEL: i64 = 3;
const CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

static ENGINE: OnceLock<Mutex<FlareEngine>> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(true);
static OVERLAY_VIEW: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static mut FLARE_VIEW_CLASS: *const Class = ptr::null();
static mut MENU_CONTROLLER_CLASS: *const Class = ptr::null();
static REGISTER_FLARE_VIEW: Once = Once::new();
static REGISTER_MENU_CONTROLLER: Once = Once::new();

pub struct PlatformApp;

impl PlatformApp {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self) {
        unsafe {
            let _pool = NSAutoreleasePool::new(nil);
            ENGINE.get_or_init(|| Mutex::new(FlareEngine::new()));

            let app = NSApp();
            app.setActivationPolicy_(NSApplicationActivationPolicyAccessory);

            let screen = NSScreen::mainScreen(nil);
            if screen == nil {
                eprintln!("popflare could not find the main screen.");
                return;
            }

            let frame = NSScreen::frame(screen);
            let window = create_overlay_window(frame);
            let view = create_flare_view(frame);
            window.setContentView_(view);
            window.makeKeyAndOrderFront_(nil);
            OVERLAY_VIEW.store(view as *mut Object, Ordering::Relaxed);

            install_status_menu();
            install_click_monitor(frame.size.height as f32);
            install_frame_timer();

            println!("popflare is running in the menu bar. Use the PF menu to toggle or quit.");
            app.run();
        }
    }
}

unsafe fn create_overlay_window(frame: NSRect) -> id {
    let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
        frame,
        NSWindowStyleMask::NSBorderlessWindowMask,
        NSBackingStoreBuffered,
        NO,
    );

    window.setOpaque_(NO);
    window.setBackgroundColor_(NSColor::clearColor(nil));
    window.setIgnoresMouseEvents_(YES);
    window.setLevel_(FLOATING_WINDOW_LEVEL);
    window.setCollectionBehavior_(NSWindowCollectionBehavior::from_bits_truncate(
        CAN_JOIN_ALL_SPACES | FULL_SCREEN_AUXILIARY,
    ));

    let _: () = msg_send![window, setReleasedWhenClosed: NO];
    window
}

unsafe fn create_flare_view(frame: NSRect) -> id {
    let class = flare_view_class();
    let view: id = msg_send![class, alloc];
    let view: id = msg_send![view, initWithFrame: frame];
    view
}


fn menubar_icon_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let candidates = [
        exe_dir
            .parent()
            .and_then(|contents| contents.parent())
            .map(|app| app.join("Contents/Resources/assets/popflare-menubar.png")),
        exe_dir
            .parent()
            .and_then(|target| target.parent())
            .map(|root| root.join("assets/popflare-menubar.png")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.exists())
        .map(|path| path.to_string_lossy().into_owned())
}

unsafe fn install_status_menu() {
    let status_item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(NSVariableStatusItemLength);
    let button: id = msg_send![status_item, button];
    if button != nil {
        if let Some(icon_path) = menubar_icon_path() {
            let ns_path = NSString::alloc(nil).init_str(&icon_path);
            let image = NSImage::alloc(nil).initWithContentsOfFile_(ns_path);

            if image != nil {
                let _: () = msg_send![image, setTemplate: YES];
                let _: () = msg_send![button, setImage: image];
            } else {
                let title = NSString::alloc(nil).init_str("PF");
                let _: () = msg_send![button, setTitle: title];
            }
        } else {
            let title = NSString::alloc(nil).init_str("PF");
            let _: () = msg_send![button, setTitle: title];
        }
    }

    let controller: id = msg_send![menu_controller_class(), new];
    let menu = NSMenu::new(nil).autorelease();

    let enabled_title = NSString::alloc(nil).init_str("Enabled");
    let enabled_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        enabled_title,
        sel!(toggleEnabled:),
        NSString::alloc(nil).init_str(""),
    );
    enabled_item.setTarget_(controller);
    let _: () = msg_send![enabled_item, setState: 1];
    menu.addItem_(enabled_item);

    let quit_title = NSString::alloc(nil).init_str("Quit Popflare");
    let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        quit_title,
        sel!(quit:),
        NSString::alloc(nil).init_str("q"),
    );
    quit_item.setTarget_(controller);
    menu.addItem_(quit_item);

    let _: () = msg_send![status_item, setMenu: menu];

    let _: () = msg_send![controller, retain];
    let _: () = msg_send![status_item, retain];
}

unsafe fn install_click_monitor(screen_height: f32) {
    let block = ConcreteBlock::new(move |event: id| {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }

        let location: NSPoint = unsafe { msg_send![event, locationInWindow] };
        let origin = Point {
            x: location.x as f32,
            y: screen_height - location.y as f32,
        };

        if let Some(engine) = ENGINE.get() {
            if let Ok(mut engine) = engine.lock() {
                engine.burst(origin);
            }
        }

        request_redraw();
    })
    .copy();

    let _: id = msg_send![class!(NSEvent), addGlobalMonitorForEventsMatchingMask: LEFT_MOUSE_DOWN_MASK handler: &*block];
    std::mem::forget(block);
}

unsafe fn install_frame_timer() {
    let block = ConcreteBlock::new(move |_timer: id| {
        if let Some(engine) = ENGINE.get() {
            if let Ok(mut engine) = engine.lock() {
                engine.update(1.0 / 60.0);
            }
        }

        request_redraw();
    })
    .copy();

    let _: id = msg_send![class!(NSTimer), scheduledTimerWithTimeInterval: 1.0f64 / 60.0f64 repeats: YES block: &*block];
    std::mem::forget(block);
}

fn request_redraw() {
    let view = OVERLAY_VIEW.load(Ordering::Relaxed);
    if !view.is_null() {
        unsafe {
            let _: () = msg_send![view, setNeedsDisplay: YES];
        }
    }
}

unsafe fn flare_view_class() -> *const Class {
    REGISTER_FLARE_VIEW.call_once(|| {
        let superclass = class!(NSView);
        let mut decl = ClassDecl::new("PopflareView", superclass).expect("PopflareView class");

        decl.add_method(
            sel!(isFlipped),
            is_flipped as extern "C" fn(&Object, Sel) -> bool,
        );
        decl.add_method(
            sel!(drawRect:),
            draw_rect as extern "C" fn(&Object, Sel, NSRect),
        );

        unsafe {
            FLARE_VIEW_CLASS = decl.register();
        }
    });

    FLARE_VIEW_CLASS
}

unsafe fn menu_controller_class() -> *const Class {
    REGISTER_MENU_CONTROLLER.call_once(|| {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("PopflareMenuController", superclass)
            .expect("PopflareMenuController class");

        decl.add_method(
            sel!(toggleEnabled:),
            toggle_enabled as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(sel!(quit:), quit as extern "C" fn(&Object, Sel, id));

        unsafe {
            MENU_CONTROLLER_CLASS = decl.register();
        }
    });

    MENU_CONTROLLER_CLASS
}

extern "C" fn toggle_enabled(_this: &Object, _cmd: Sel, item: id) {
    let next = !ENABLED.load(Ordering::Relaxed);
    ENABLED.store(next, Ordering::Relaxed);

    unsafe {
        let _: () = msg_send![item, setState: if next { 1 } else { 0 }];
    }
}

extern "C" fn quit(_this: &Object, _cmd: Sel, _item: id) {
    unsafe {
        let app = NSApp();
        let _: () = msg_send![app, terminate: nil];
    }
}

extern "C" fn is_flipped(_this: &Object, _cmd: Sel) -> bool {
    true
}

extern "C" fn draw_rect(_this: &Object, _cmd: Sel, _rect: NSRect) {
    let Some(engine) = ENGINE.get() else {
        return;
    };

    let Ok(engine) = engine.lock() else {
        return;
    };

    unsafe {
        for particle in engine.particles() {
            let color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                nil,
                particle.color.r as f64,
                particle.color.g as f64,
                particle.color.b as f64,
                particle.color.a as f64,
            );
            let _: () = msg_send![color, set];

            let radius = particle.radius as f64;
            let rect = NSRect::new(
                NSPoint::new(
                    particle.position.x as f64 - radius,
                    particle.position.y as f64 - radius,
                ),
                NSSize::new(radius * 2.0, radius * 2.0),
            );

            let path: id = msg_send![class!(NSBezierPath), bezierPathWithOvalInRect: rect];
            let _: () = msg_send![path, fill];
        }
    }
}
