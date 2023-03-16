fn main() {
    // Avoid unnecessary re-building.
    println!("cargo:rerun-if-changed=build.rs");
}
