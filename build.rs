extern crate bindgen;
extern crate cmake;

use std::env;
use std::fs::create_dir;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn main() {
    #[cfg(any(feature = "exif"))]
    {
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let utils_build_path = out_path.join("utils");
        if !utils_build_path.exists() {
            create_dir(&utils_build_path).unwrap();
        }
        cmake::Config::new("utils")
            .define("CMAKE_INSTALL_PREFIX", out_path.to_str().unwrap())
            .out_dir(utils_build_path)
            .define("ENABLE_ICONV", "OFF")
            .define("ENABLE_STANDALONE", "ON")
            .define("ENABLE_CXX17", "ON")
            .define("CMAKE_BUILD_TYPE", "Release")
            .define("INSTALL_DEP_FILES", "ON")
            .build();
        let dep_path = out_path.join("utils_dep.txt");
        let mut f = File::open(dep_path).unwrap();
        let mut s = String::from("");
        f.read_to_string(&mut s).unwrap();
        println!("cargo:rustc-link-lib=static=exif");
        let l: Vec<&str> = s.split(";").collect();
        for i in l.iter() {
            let mut p = PathBuf::from(i);
            let p2 = p.clone();
            let file_name = p2.file_stem().unwrap();
            let file_name = file_name.to_str().unwrap();
            p.pop();
            let pa = p.to_str().unwrap();
            if pa != "" {
                println!("cargo:rustc-link-search={}", pa);
            }
            println!("cargo:rustc-link-lib={}", file_name);
        }
        println!("cargo:rerun-if-changed=utils/");
    }
    #[cfg(feature = "exif")]
    {
        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        let exif_build_path = out_path.join("exif");
        let utils_build_path = out_path.join("utils");
        if !exif_build_path.exists() {
            create_dir(&exif_build_path).unwrap();
        }
        cmake::Config::new("exif")
            .define("CMAKE_INSTALL_PREFIX", out_path.to_str().unwrap())
            .out_dir(exif_build_path)
            .define("UTILS_LIBRARY", utils_build_path.to_str().unwrap())
            .define("CMAKE_BUILD_TYPE", "Release")
            .build();
        println!("cargo:rustc-link-search=native={}/lib", out_path.display());
        let dep_path = out_path.join("exif_dep.txt");
        let mut f = File::open(dep_path).unwrap();
        let mut s = String::from("");
        f.read_to_string(&mut s).unwrap();
        println!("cargo:rustc-link-lib=static=exif");
        let l: Vec<&str> = s.split(";").collect();
        for i in l.iter() {
            let mut p = PathBuf::from(i);
            let p2 = p.clone();
            let file_name = p2.file_stem().unwrap();
            let file_name = file_name.to_str().unwrap();
            p.pop();
            println!("cargo:rustc-link-search={}", p.to_str().unwrap());
            println!("cargo:rustc-link-lib={}", file_name);
        }
        println!("cargo:rerun-if-changed=exif/");
        let bindings = bindgen::Builder::default()
            // The input header we would like to generate
            // bindings for.
            .header("exif/exif.h")
            // Tell cargo to invalidate the built crate whenever any of the
            // included header files changed.
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            // Finish the builder and generate the bindings.
            .generate()
            // Unwrap the Result and panic on failure.
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("exif.rs"))
            .expect("Couldn't write bindings!");
    }
}
