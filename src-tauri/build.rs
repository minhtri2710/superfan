fn main() {
    #[cfg(target_os = "macos")]
    {
        // Compile smc library for main app FFI
        cc::Build::new()
            .file("src/smc/smc.c")
            .include("src/smc")
            .warnings(false)
            .compile("smc");
        println!("cargo:rustc-link-lib=framework=IOKit");

        // Compile standalone smc-helper binary
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let status = std::process::Command::new("clang")
            .args(&[
                "-O2",
                "-framework",
                "IOKit",
                "-framework",
                "CoreFoundation",
                "src/smc/smc.c",
                "-o",
                &format!("{}/smc-helper", out_dir),
            ])
            .status();

        if let Ok(st) = status {
            if st.success() {
                println!("cargo:rerun-if-changed=src/smc/smc.c");
            }
        }
    }

    tauri_build::build();
}
