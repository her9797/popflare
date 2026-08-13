#![allow(deprecated, unexpected_cfgs, unsafe_op_in_unsafe_fn)]

use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering};
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

use crate::effect::{EffectStyle, FlareEngine, Point, SparkleKind};

const LEFT_MOUSE_DOWN_MASK: u64 = 1 << 1;
const FLOATING_WINDOW_LEVEL: i64 = 3;
const CAN_JOIN_ALL_SPACES: u64 = 1 << 0;
const FULL_SCREEN_AUXILIARY: u64 = 1 << 8;

static ENGINE: OnceLock<Mutex<FlareEngine>> = OnceLock::new();
static ENABLED: AtomicBool = AtomicBool::new(true);
static EFFECT_STYLE: AtomicUsize = AtomicUsize::new(0);
static STATUS_BUTTON: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static COLOR_BURST_ITEM: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static COLOR_RINGS_ITEM: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static PINK_SPARKLES_ITEM: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static COLOR_SPARKLES_ITEM: AtomicPtr<Object> = AtomicPtr::new(ptr::null_mut());
static MENU_ICON_TICK: AtomicUsize = AtomicUsize::new(0);
static MENU_ICON_PHASE: AtomicUsize = AtomicUsize::new(0);
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

            let Some(frame) = virtual_screen_frame() else {
                eprintln!("popflare could not find any screen.");
                return;
            };

            let window = create_overlay_window(frame);
            let view_frame = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
            let view = create_flare_view(view_frame);
            window.setContentView_(view);
            window.makeKeyAndOrderFront_(nil);
            OVERLAY_VIEW.store(view as *mut Object, Ordering::Relaxed);

            install_status_menu();
            install_click_monitor(frame);
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

unsafe fn virtual_screen_frame() -> Option<NSRect> {
    let screens = NSScreen::screens(nil);
    if screens == nil {
        return None;
    }

    let count: usize = msg_send![screens, count];
    if count == 0 {
        return None;
    }

    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for index in 0..count {
        let screen: id = msg_send![screens, objectAtIndex: index];
        if screen == nil {
            continue;
        }

        let frame = NSScreen::frame(screen);
        min_x = min_x.min(frame.origin.x);
        min_y = min_y.min(frame.origin.y);
        max_x = max_x.max(frame.origin.x + frame.size.width);
        max_y = max_y.max(frame.origin.y + frame.size.height);
    }

    if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
        return None;
    }

    Some(NSRect::new(
        NSPoint::new(min_x, min_y),
        NSSize::new(max_x - min_x, max_y - min_y),
    ))
}



fn asset_path(filename: &str) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;
    let candidates = [
        exe_dir
            .parent()
            .and_then(|contents| contents.parent())
            .map(|app| app.join("Contents/Resources/assets").join(filename)),
        exe_dir
            .parent()
            .and_then(|target| target.parent())
            .map(|root| root.join("assets").join(filename)),
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
        if install_animated_menu_icon(button) {
            STATUS_BUTTON.store(button as *mut Object, Ordering::Relaxed);
        } else {
            let title = NSString::alloc(nil).init_str("PF");
            let _: () = msg_send![button, setTitle: title];
        }
    }
    STATUS_BUTTON.store(button as *mut Object, Ordering::Relaxed);

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

    menu.addItem_(NSMenuItem::separatorItem(nil));

    let color_title = NSString::alloc(nil).init_str("Color Burst");
    let color_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        color_title,
        sel!(selectColorBurst:),
        NSString::alloc(nil).init_str(""),
    );
    color_item.setTarget_(controller);
    let _: () = msg_send![color_item, setState: 1];
    menu.addItem_(color_item);
    COLOR_BURST_ITEM.store(color_item as *mut Object, Ordering::Relaxed);

    let cursor_title = NSString::alloc(nil).init_str("Rainbow Circle");
    let cursor_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        cursor_title,
        sel!(selectColorRings:),
        NSString::alloc(nil).init_str(""),
    );
    cursor_item.setTarget_(controller);
    menu.addItem_(cursor_item);
    COLOR_RINGS_ITEM.store(cursor_item as *mut Object, Ordering::Relaxed);

    let pink_title = NSString::alloc(nil).init_str("Pink Sparkles");
    let pink_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        pink_title,
        sel!(selectPinkSparkles:),
        NSString::alloc(nil).init_str(""),
    );
    pink_item.setTarget_(controller);
    menu.addItem_(pink_item);
    PINK_SPARKLES_ITEM.store(pink_item as *mut Object, Ordering::Relaxed);

    let color_sparkles_title = NSString::alloc(nil).init_str("Color Sparkles");
    let color_sparkles_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
        color_sparkles_title,
        sel!(selectColorSparkles:),
        NSString::alloc(nil).init_str(""),
    );
    color_sparkles_item.setTarget_(controller);
    menu.addItem_(color_sparkles_item);
    COLOR_SPARKLES_ITEM.store(color_sparkles_item as *mut Object, Ordering::Relaxed);


    menu.addItem_(NSMenuItem::separatorItem(nil));

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

unsafe fn install_click_monitor(screen_frame: NSRect) {
    let block = ConcreteBlock::new(move |event: id| {
        if !ENABLED.load(Ordering::Relaxed) {
            return;
        }

        let location: NSPoint = unsafe { msg_send![event, locationInWindow] };
        let origin = Point {
            x: (location.x - screen_frame.origin.x) as f32,
            y: (screen_frame.origin.y + screen_frame.size.height - location.y) as f32,
        };

        let style = selected_effect_style();
        if let Some(engine) = ENGINE.get() {
            if let Ok(mut engine) = engine.lock() {
                engine.burst(origin, style);
            }
        }

        request_redraw();
    })
    .copy();

    let _: id = msg_send![class!(NSEvent), addGlobalMonitorForEventsMatchingMask: LEFT_MOUSE_DOWN_MASK handler: &*block];
    std::mem::forget(block);
}

unsafe fn install_animated_menu_icon(button: id) -> bool {
    let image = rotated_menu_png(0.0);
    if image == nil {
        return false;
    }

    let _: () = msg_send![button, setImage: image];
    true
}

fn update_menu_icon_rotation() {
    let tick = MENU_ICON_TICK.fetch_add(1, Ordering::Relaxed);
    if tick % 6 != 0 {
        return;
    }

    let button = STATUS_BUTTON.load(Ordering::Relaxed);
    if button.is_null() {
        return;
    }

    let phase = MENU_ICON_PHASE.fetch_add(1, Ordering::Relaxed) % 20;
    let degrees = phase as f64 * 18.0;

    unsafe {
        let image = rotated_menu_png(degrees);
        if image != nil {
            let _: () = msg_send![button, setImage: image];
        }
    }
}

unsafe fn rotated_menu_png(degrees: f64) -> id {
    let Some(path) = asset_path("popflare-menubar.png") else {
        return nil;
    };

    let ns_path = NSString::alloc(nil).init_str(&path);
    let source = NSImage::alloc(nil).initWithContentsOfFile_(ns_path);
    if source == nil {
        return nil;
    }

    let size = NSSize::new(18.0, 18.0);
    let image = NSImage::alloc(nil).initWithSize_(size);
    let _: () = msg_send![image, lockFocus];

    let transform: id = msg_send![class!(NSAffineTransform), transform];
    let _: () = msg_send![transform, translateXBy: 9.0f64 yBy: 9.0f64];
    let _: () = msg_send![transform, rotateByDegrees: degrees];
    let _: () = msg_send![transform, translateXBy: -9.0f64 yBy: -9.0f64];
    let _: () = msg_send![transform, concat];

    let rect = NSRect::new(NSPoint::new(0.0, 0.0), size);
    let _: () = msg_send![source, drawInRect: rect];
    let _: () = msg_send![image, unlockFocus];
    let _: () = msg_send![image, setTemplate: YES];
    let _: () = msg_send![image, setSize: size];
    image
}

unsafe fn install_frame_timer() {
    let block = ConcreteBlock::new(move |_timer: id| {
        if let Some(engine) = ENGINE.get() {
            if let Ok(mut engine) = engine.lock() {
                engine.update(1.0 / 60.0);
            }
        }

        update_menu_icon_rotation();
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
        decl.add_method(
            sel!(selectColorBurst:),
            select_color_burst as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(selectColorRings:),
            select_color_rings as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(selectPinkSparkles:),
            select_pink_sparkles as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(selectColorSparkles:),
            select_color_sparkles as extern "C" fn(&Object, Sel, id),
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


fn selected_effect_style() -> EffectStyle {
    match EFFECT_STYLE.load(Ordering::Relaxed) {
        1 => EffectStyle::ColorRings,
        2 => EffectStyle::PinkSparkles,
        3 => EffectStyle::ColorSparkles,
        _ => EffectStyle::ColorBurst,
    }
}

fn set_effect_style(style: EffectStyle) {
    let style_index = match style {
        EffectStyle::ColorBurst => 0,
        EffectStyle::ColorRings => 1,
        EffectStyle::PinkSparkles => 2,
        EffectStyle::ColorSparkles => 3,
    };
    EFFECT_STYLE.store(style_index, Ordering::Relaxed);

    unsafe {
        let color_item = COLOR_BURST_ITEM.load(Ordering::Relaxed);
        if !color_item.is_null() {
            let _: () = msg_send![color_item, setState: if style == EffectStyle::ColorBurst { 1 } else { 0 }];
        }

        let cursor_item = COLOR_RINGS_ITEM.load(Ordering::Relaxed);
        if !cursor_item.is_null() {
            let _: () = msg_send![cursor_item, setState: if style == EffectStyle::ColorRings { 1 } else { 0 }];
        }

        let pink_item = PINK_SPARKLES_ITEM.load(Ordering::Relaxed);
        if !pink_item.is_null() {
            let _: () = msg_send![pink_item, setState: if style == EffectStyle::PinkSparkles { 1 } else { 0 }];
        }

        let color_sparkles_item = COLOR_SPARKLES_ITEM.load(Ordering::Relaxed);
        if !color_sparkles_item.is_null() {
            let _: () = msg_send![color_sparkles_item, setState: if style == EffectStyle::ColorSparkles { 1 } else { 0 }];
        }

    }
}

extern "C" fn select_color_burst(_this: &Object, _cmd: Sel, _item: id) {
    set_effect_style(EffectStyle::ColorBurst);
}

extern "C" fn select_color_rings(_this: &Object, _cmd: Sel, _item: id) {
    set_effect_style(EffectStyle::ColorRings);
}

extern "C" fn select_pink_sparkles(_this: &Object, _cmd: Sel, _item: id) {
    set_effect_style(EffectStyle::PinkSparkles);
}

extern "C" fn select_color_sparkles(_this: &Object, _cmd: Sel, _item: id) {
    set_effect_style(EffectStyle::ColorSparkles);
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


unsafe fn draw_sparkle(origin: Point, kind: SparkleKind, size: f32, rotation: f32, color: crate::effect::Color, opacity: f32) {
    let color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
        nil,
        color.r as f64,
        color.g as f64,
        color.b as f64,
        opacity as f64,
    );
    let _: () = msg_send![color, set];

    match kind {
        SparkleKind::Plus => draw_plus(origin, size, rotation),
        SparkleKind::Diamond => draw_diamond(origin, size, rotation),
        SparkleKind::Star => draw_star(origin, size, rotation),
        SparkleKind::Twinkle => draw_twinkle(origin, size, rotation),
        SparkleKind::Dot => draw_dot(origin, size),
        SparkleKind::Asterisk => draw_asterisk(origin, size, rotation),
    }
}

unsafe fn draw_dot(origin: Point, size: f32) {
    let radius = size as f64;
    let rect = NSRect::new(
        NSPoint::new(origin.x as f64 - radius, origin.y as f64 - radius),
        NSSize::new(radius * 2.0, radius * 2.0),
    );
    let path: id = msg_send![class!(NSBezierPath), bezierPathWithOvalInRect: rect];
    let _: () = msg_send![path, fill];
}

unsafe fn draw_asterisk(origin: Point, size: f32, rotation: f32) {
    let path: id = msg_send![class!(NSBezierPath), bezierPath];
    let _: () = msg_send![path, setLineWidth: (size * 0.14) as f64];
    for index in 0..3 {
        let angle = rotation + index as f32 * std::f32::consts::PI / 3.0;
        let dx = angle.cos() * size * 0.48;
        let dy = angle.sin() * size * 0.48;
        draw_segment(path, origin, -dx, -dy, dx, dy, 0.0);
    }
    let _: () = msg_send![path, stroke];
}

unsafe fn draw_plus(origin: Point, size: f32, rotation: f32) {
    let path: id = msg_send![class!(NSBezierPath), bezierPath];
    let _: () = msg_send![path, setLineWidth: (size * 0.18) as f64];
    draw_segment(path, origin, -size * 0.55, 0.0, size * 0.55, 0.0, rotation);
    draw_segment(path, origin, 0.0, -size * 0.55, 0.0, size * 0.55, rotation);
    let _: () = msg_send![path, stroke];
}

unsafe fn draw_segment(path: id, origin: Point, x1: f32, y1: f32, x2: f32, y2: f32, rotation: f32) {
    let start = rotated_point(origin, x1, y1, rotation);
    let end = rotated_point(origin, x2, y2, rotation);
    let _: () = msg_send![path, moveToPoint: start];
    let _: () = msg_send![path, lineToPoint: end];
}

unsafe fn draw_diamond(origin: Point, size: f32, rotation: f32) {
    let points = [
        (0.0, -size * 0.62),
        (size * 0.28, 0.0),
        (0.0, size * 0.62),
        (-size * 0.28, 0.0),
    ];
    stroke_polygon(origin, &points, rotation, size * 0.16);
}

unsafe fn draw_twinkle(origin: Point, size: f32, rotation: f32) {
    let points = [
        (0.0, -size * 0.98),
        (size * 0.17, -size * 0.17),
        (size * 0.98, 0.0),
        (size * 0.17, size * 0.17),
        (0.0, size * 0.98),
        (-size * 0.17, size * 0.17),
        (-size * 0.98, 0.0),
        (-size * 0.17, -size * 0.17),
    ];
    fill_polygon(origin, &points, rotation);
}

unsafe fn draw_star(origin: Point, size: f32, rotation: f32) {
    let points = [
        (0.0, -size * 0.62),
        (size * 0.16, -size * 0.18),
        (size * 0.54, -size * 0.10),
        (size * 0.24, size * 0.14),
        (size * 0.34, size * 0.54),
        (0.0, size * 0.30),
        (-size * 0.34, size * 0.54),
        (-size * 0.24, size * 0.14),
        (-size * 0.54, -size * 0.10),
        (-size * 0.16, -size * 0.18),
    ];
    fill_polygon(origin, &points, rotation);
}

unsafe fn stroke_polygon(origin: Point, points: &[(f32, f32)], rotation: f32, line_width: f32) {
    let path: id = msg_send![class!(NSBezierPath), bezierPath];
    let _: () = msg_send![path, setLineWidth: line_width as f64];
    for (index, (x, y)) in points.iter().enumerate() {
        let point = rotated_point(origin, *x, *y, rotation);
        if index == 0 {
            let _: () = msg_send![path, moveToPoint: point];
        } else {
            let _: () = msg_send![path, lineToPoint: point];
        }
    }
    let _: () = msg_send![path, closePath];
    let _: () = msg_send![path, stroke];
}

unsafe fn fill_polygon(origin: Point, points: &[(f32, f32)], rotation: f32) {
    let path: id = msg_send![class!(NSBezierPath), bezierPath];
    for (index, (x, y)) in points.iter().enumerate() {
        let point = rotated_point(origin, *x, *y, rotation);
        if index == 0 {
            let _: () = msg_send![path, moveToPoint: point];
        } else {
            let _: () = msg_send![path, lineToPoint: point];
        }
    }
    let _: () = msg_send![path, closePath];
    let _: () = msg_send![path, fill];
}

fn rotated_point(origin: Point, x: f32, y: f32, rotation: f32) -> NSPoint {
    let cos = rotation.cos();
    let sin = rotation.sin();
    NSPoint::new(
        origin.x as f64 + (x * cos - y * sin) as f64,
        origin.y as f64 + (x * sin + y * cos) as f64,
    )
}

unsafe fn draw_color_rings(origin: Point, scale: f32, opacity: f32) {
    let radius = 27.0 * scale;
    let glow = [
        (1.42, 7.2, 1.00, 0.58, 0.28, 0.42),
        (1.26, 6.4, 1.00, 0.82, 0.22, 0.52),
        (1.10, 6.1, 0.34, 0.96, 0.48, 0.56),
        (0.94, 5.8, 0.22, 0.76, 1.00, 0.58),
        (0.78, 5.4, 0.72, 0.42, 1.00, 0.54),
        (0.62, 5.0, 1.00, 0.45, 0.90, 0.48),
    ];

    for (ring_scale, line_width, red, green, blue, alpha) in glow {
        let color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
            nil,
            red,
            green,
            blue,
            (opacity * alpha) as f64,
        );
        draw_ellipse_ring(
            origin,
            radius * ring_scale,
            radius * ring_scale,
            line_width * scale,
            color,
        );
    }

    let soft_white = NSColor::colorWithCalibratedRed_green_blue_alpha_(
        nil,
        1.0,
        1.0,
        1.0,
        (opacity * 0.34) as f64,
    );
    draw_ellipse_ring(origin, radius * 0.48, radius * 0.48, 2.8 * scale, soft_white);
}

unsafe fn draw_ellipse_ring(origin: Point, width_radius: f32, height_radius: f32, line_width: f32, color: id) {
    let _: () = msg_send![color, set];
    let rect = NSRect::new(
        NSPoint::new(
            origin.x as f64 - width_radius as f64,
            origin.y as f64 - height_radius as f64,
        ),
        NSSize::new(width_radius as f64 * 2.0, height_radius as f64 * 2.0),
    );
    let path: id = msg_send![class!(NSBezierPath), bezierPathWithOvalInRect: rect];
    let _: () = msg_send![path, setLineWidth: line_width as f64];
    let _: () = msg_send![path, stroke];
}


extern "C" fn draw_rect(_this: &Object, _cmd: Sel, _rect: NSRect) {
    let Some(engine) = ENGINE.get() else {
        return;
    };

    let Ok(engine) = engine.lock() else {
        return;
    };

    unsafe {
        for ring in engine.color_rings() {
            draw_color_rings(ring.origin, ring.scale, ring.opacity);
        }

        for sparkle in engine.sparkles() {
            draw_sparkle(sparkle.position, sparkle.kind, sparkle.size, sparkle.rotation, sparkle.color, sparkle.color.a);
        }

        for particle in engine.particles() {
            let color = NSColor::colorWithCalibratedRed_green_blue_alpha_(
                nil,
                particle.color.r as f64,
                particle.color.g as f64,
                particle.color.b as f64,
                particle.color.a as f64,
            );
            let _: () = msg_send![color, set];

            let half = particle.length as f64 / 2.0;
            let dx = particle.angle.cos() as f64 * half;
            let dy = particle.angle.sin() as f64 * half;
            let start = NSPoint::new(
                particle.position.x as f64 - dx,
                particle.position.y as f64 - dy,
            );
            let end = NSPoint::new(
                particle.position.x as f64 + dx,
                particle.position.y as f64 + dy,
            );

            let path: id = msg_send![class!(NSBezierPath), bezierPath];
            let _: () = msg_send![path, setLineWidth: particle.radius as f64];
            let _: () = msg_send![path, moveToPoint: start];
            let _: () = msg_send![path, lineToPoint: end];
            let _: () = msg_send![path, stroke];
        }
    }
}
