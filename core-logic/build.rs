fn main() {
    let mut build = cxx_build::bridge("src/main.rs");
    build.file("src/preview.cpp");
    
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "msvc" {
        build.flag("/std:c++17");
    } else {
        build.flag_if_supported("-std=c++17");
    }
    
    build.compile("urglance");

    slint_build::compile("ui/appwindow.slint").unwrap();

    println!("cargo:rerun-if-changed=ui/appwindow.slint");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/preview.cpp");
    println!("cargo:rerun-if-changed=src/preview.h");
}
