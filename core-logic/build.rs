fn main() {
    cxx_build::bridge("src/main.rs")
        .file("src/preview.cpp")
        .flag_if_supported("-std=c++17")
        .compile("urfileorganizer");

    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=src/preview.cpp");
    println!("cargo:rerun-if-changed=src/preview.h");
}
