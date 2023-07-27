use std::{env, path::PathBuf, process::Command};

use bindgen::CargoCallbacks;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("cp")
        .arg("-r")
        .arg("lib")
        .arg(out_dir.clone())
        .status()
        .expect("copy liburing to out_dir");
    Command::new("make")
        .arg("liburing.a")
        .current_dir(format!("{}/lib/src", out_dir.clone()))
        .env("CFLAGS", "-fPIC")
        .status()
        .expect("failed to build liburing.a");

    // Tell cargo to tell rustc to link the system liburing
    // shared library.
    println!("cargo:rustc-link-lib=uring");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-search=native={}/lib/src", out_dir.clone());

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(CargoCallbacks))
        .allowlist_function("__io_uring.*")
        .allowlist_function("io_uring.*")
        .allowlist_var("IORING.*")
        .allowlist_var("IOSQE.*")
        .allowlist_type("io_uring.*")
        .prepend_enum_name(false)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(out_dir);
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
