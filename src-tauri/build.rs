fn main() {
    #[cfg(target_os = "macos")]
    {
        // Compile smc library for main app FFI
        cc::Build::new()
            .file("src/smc/smc.c")
            .file("src/smc/power_events.m")
            .include("src/smc")
            .define("SUPERFAN_SMC_LIBRARY", None)
            .warnings(false)
            .compile("smc");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");

        println!("cargo:rerun-if-changed=src/smc/smc.c");
        println!("cargo:rerun-if-changed=src/smc/power_events.m");
    }

    tauri_build::build();
}
