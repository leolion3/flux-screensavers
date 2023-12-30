use cfg_aliases::cfg_aliases;

fn env(name: &'static str) -> String {
    std::env::var(name).unwrap_or_default()
}

fn main() {
    // Setup alias to reduce `cfg` boilerplate.
    cfg_aliases! {
        // Systems.
        android_platform: { target_os = "android" },
        wasm_platform: { target_family = "wasm" },
        macos_platform: { target_os = "macos" },
        ios_platform: { target_os = "ios" },
        apple: { any(ios_platform, macos_platform) },
        free_unix: { all(unix, not(apple), not(android_platform)) },

        // Native displays.
        x11_platform: { all(free_unix, not(wasm_platform)) },
        wayland_platform: { all(free_unix, not(wasm_platform)) },

        // Backends.
        egl_backend: { all(any(windows, unix), not(apple), not(wasm_platform)) },
        glx_backend: { all(x11_platform, not(wasm_platform)) },
        wgl_backend: { all(windows, not(wasm_platform)) },
        cgl_backend: { all(macos_platform, not(wasm_platform)) },
    }

    // Run windres only when building releases for Windows.
    if env("CARGO_CFG_TARGET_OS") != "windows" {
        return;
    }

    // Skip windres in development.
    if env("PROFILE") == "release" {
        let mut resource = winres::WindowsResource::new();

        // If cross-compiling, use the correct tool names. These should
        // already be in our path on NixOS. In case they’re not, you can
        // also set `toolkit_path`.
        //
        // Here’s where this stuff is on
        // NixOS: pkgs.pkgsCross.mingwW64.stdenv.cc.bintools.bintools_bin
        if cfg!(unix) {
            resource
                .set_ar_path("x86_64-w64-mingw32-ar")
                .set_windres_path("x86_64-w64-mingw32-windres");
        }

        resource
            .set_icon("installer/flux-screensaver.ico")
            .set_manifest_file("flux-screensaver-windows.exe.manifest");

        if let Err(msg) = resource.compile() {
            eprintln!("Couldn’t compile the Windows resource:\n{}", msg);
            std::process::exit(1);
        }
    }
}
