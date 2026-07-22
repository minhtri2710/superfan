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

        cc::Build::new()
            .file("src/fan_actuation/service_management.m")
            .warnings(false)
            .compile("fan_service_management");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=ServiceManagement");

        println!("cargo:rerun-if-changed=src/smc/smc.c");
        println!("cargo:rerun-if-changed=src/fan_actuation/service_management.m");
    }

    tauri_build::build();
}
