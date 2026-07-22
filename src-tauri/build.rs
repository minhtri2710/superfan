fn main() {
    #[cfg(target_os = "macos")]
    {
        // Compile smc library for main app FFI
        cc::Build::new()
            .file("src/smc/smc.c")
            .include("src/smc")
            .define("SUPERFAN_SMC_LIBRARY", None)
            .warnings(false)
            .compile("smc");
        println!("cargo:rustc-link-lib=framework=IOKit");

        println!("cargo:rerun-if-changed=src/smc/smc.c");
    }

    tauri_build::build();
}
