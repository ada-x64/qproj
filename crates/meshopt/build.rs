//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
// Forked from https://github.com/gwihlidal/meshopt-rs
use std::env;
fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let xwin_dir = format!("{manifest_dir}/../../.xwin-cache/splat",);
    let msvc_flags = [
        "-Wno-unused-command-line-argument",
        "-fuse-ld=lld-link",
        "/std:11",
        "/TP",
        &format!("/imsvc{xwin_dir}/crt/include"),
        &format!("/imsvc{xwin_dir}/sdk/include/ucrt"),
        &format!("/imsvc{xwin_dir}/sdk/include/um"),
        &format!("/imsvc{xwin_dir}/sdk/include/shared"),
    ];

    let mut build = cc::Build::new();

    build.include("src");

    // // Add the files we build
    // let source_files = ["meshoptimizer/src/simplifier.cpp"];
    // Add the files we build
    let source_files = [
        "meshoptimizer/src/allocator.cpp",
        "meshoptimizer/src/clusterizer.cpp",
        "meshoptimizer/src/indexcodec.cpp",
        "meshoptimizer/src/indexgenerator.cpp",
        "meshoptimizer/src/overdrawoptimizer.cpp",
        "meshoptimizer/src/simplifier.cpp",
        "meshoptimizer/src/spatialorder.cpp",
        "meshoptimizer/src/stripifier.cpp",
        "meshoptimizer/src/vcacheoptimizer.cpp",
        "meshoptimizer/src/vertexcodec.cpp",
        "meshoptimizer/src/vfetchoptimizer.cpp",
    ];

    for source_file in &source_files {
        build.file(source_file);
    }

    let target = env::var("TARGET").unwrap();
    if target.contains("darwin") {
        build
            .flag("-std=c++11")
            .cpp_link_stdlib("c++")
            .cpp_set_stdlib("c++")
            .cpp(true);
    } else if target.contains("linux") || target.contains("windows-gnu") {
        build.cpp(true);
    } else if target.contains("windows-msvc") {
        msvc_flags.iter().for_each(|f| {
            build.flag(f);
        });
    }

    build.compile("meshopt_cpp");

    let mut bindings = bindgen::Builder::default()
        .header("meshoptimizer/src/meshoptimizer.h")
        .derive_debug(true)
        .impl_debug(true)
        .blocklist_type("__darwin_.*")
        .allowlist_function("meshopt.*")
        .trust_clang_mangling(false)
        .layout_tests(false)
        .size_t_is_usize(true)
        .emit_diagnostics();

    if target.contains("windows-msvc") {
        let msvc_flags = [
            "-Wno-unused-command-line-argument",
            "-fuse-ld=lld-link",
            "/std:11",
            "/TP",
            &format!("-isystem{xwin_dir}/crt/include"),
            &format!("-isystem{xwin_dir}/sdk/include/ucrt"),
            &format!("-isystem{xwin_dir}/sdk/include/um"),
            &format!("-isystem{xwin_dir}/sdk/include/shared"),
        ];
        bindings = bindings.clang_args(msvc_flags);
    }

    let bindings = bindings.generate().expect("Unable to generate bindings!");

    bindings
        .write_to_file(std::path::Path::new("gen/bindings.rs"))
        .expect("Unable to write bindings!");
}
