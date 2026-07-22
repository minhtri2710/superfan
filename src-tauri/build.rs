fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/smc/smc.c")
            .include("src/smc")
            .warnings(false)
            .compile("smc");
        println!("cargo:rustc-link-lib=framework=IOKit");
    }

    tauri_build::build();
}
