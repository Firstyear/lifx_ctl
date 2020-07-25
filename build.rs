fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=templates/base.html");
    println!("cargo:rerun-if-changed=templates/status.html");
    println!("cargo:rerun-if-changed=templates/manual.html");
}
