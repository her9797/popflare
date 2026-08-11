# popflare

A tiny macOS click effect app. Every click pops a small firework.

## Goal

popflare listens for global mouse clicks and renders a short firework burst at the click position using a transparent click-through overlay window.

## v0.1 Shape

- Pure Rust flare particle engine
- macOS-only platform layer
- Next step: global click detection with `CGEventTap`
- Next step: transparent always-on-top `NSWindow`
- Next step: render particles on the overlay without intercepting clicks

## Current State

The project is scaffolded and the particle engine exists. The native macOS event tap and overlay window are intentionally isolated under `src/platform/macos.rs` so they can be implemented without leaking platform details into the effect engine.
