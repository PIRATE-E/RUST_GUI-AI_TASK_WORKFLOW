// Slint build script
// WHAT: Compiles .slint UI markup into Rust structs and traits at build time.
// WHY: Slint code-generates Rust types in OUT_DIR so Rust can safely typecheck UI properties and callbacks.

fn main() {
    slint_build::compile("ui/app-window.slint").expect("Failed to compile Slint UI definition");
}
