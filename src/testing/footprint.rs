//! AtomVM Size Testing
//! 
//! Simple size tests to ensure avmgl-rs stays within AtomVM constraints.
//! These tests measure the actual no_std production builds.

#[cfg(test)]
mod tests {
    use std::process::Command;
    use std::fs;

    // ═══════════════════════════════════════════════════════════════════════════
    // 🎛️  SIZE CONTROL KNOBS - Adjust these to tune AtomVM footprint limits
    // ═══════════════════════════════════════════════════════════════════════════
    
    // 📊 BASELINE SIZE LIMIT: Core driver without any AtomVM integration
    // Controls: no_std, trait abstractions, display logic complexity
    // Reduce by: simplifying traits, removing unused display features, const generics
    const MAX_BASE_SIZE_KB: f64 = 173.0;    
    
    // 📊 NIF SIZE LIMIT: Full library including AtomVM bindings  
    // Controls: avmnif-rs dependency, NIF function count, term conversion overhead
    // Reduce by: fewer NIF exports, simpler term handling, feature-gated NIFs
    const MAX_NIF_SIZE_KB: f64 = 192.0;     

    #[test]
    fn test_base_library_size() {
        println!("Building no_std base library for size measurement...");
        
        // 🏗️  SIZE LEVER: Build configuration affects final size
        // --release: Optimizations reduce size significantly  
        // --no-default-features: Excludes avmnif-rs dependency (~100KB savings)
        // -C opt-level=z: (not used) Would optimize for size over speed
        let output = Command::new("cargo")
            .args(&["build", "--lib", "--release", "--no-default-features"])
            .env("CARGO_CFG_NOT_TEST", "1")  // 🎛️  Force no_std/no_main (prod config)
            .output()
            .expect("Failed to build base library");

        if !output.status.success() {
            println!("Build stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Build failed");
        }

        // 📏 SIZE MEASUREMENT: .rlib includes metadata but approximates final size
        let lib_path = "target/release/libavmgl_rs.rlib";
        let metadata = fs::metadata(lib_path).expect("Library file not found");
        let size_kb = metadata.len() as f64 / 1024.0;

        println!("Base library (no_std) size: {:.2} KB", size_kb);

        // 🚨 SIZE ENFORCEMENT: Fail build if library exceeds AtomVM constraints
        assert!(
            size_kb < MAX_BASE_SIZE_KB,
            "Library too large: {:.2} KB > {:.2} KB. \
             🎛️  Reduce by: simplifying traits, removing debug info, feature-gating code",
            size_kb, MAX_BASE_SIZE_KB
        );
    }

    #[test]
    fn test_nif_library_size() {
        println!("Building no_std NIF library for size measurement...");
        
        // 🏗️  SIZE LEVER: NIF features add significant overhead
        // --features nifs: Includes avmnif-rs (~50-100KB), NIF collection macros, term handling
        // Consider: conditional NIF exports, minimal term conversions, lazy static alternatives
        let output = Command::new("cargo")
            .args(&["build", "--lib", "--release", "--features", "nifs"])
            .env("CARGO_CFG_NOT_TEST", "1")  // 🎛️  Force production build settings
            .output()
            .expect("Failed to build NIF library");

        if !output.status.success() {
            println!("Build stderr: {}", String::from_utf8_lossy(&output.stderr));
            panic!("NIF build failed");
        }

        // 📏 SIZE MEASUREMENT: Full NIF-enabled library size
        let lib_path = "target/release/libavmgl_rs.rlib";
        let metadata = fs::metadata(lib_path).expect("Library file not found");
        let size_kb = metadata.len() as f64 / 1024.0;

        println!("NIF library (no_std + NIFs) size: {:.2} KB", size_kb);

        // 🚨 SIZE ENFORCEMENT: AtomVM memory constraints
        assert!(
            size_kb < MAX_NIF_SIZE_KB,
            "NIF library too large: {:.2} KB > {:.2} KB. \
             🎛️  Reduce by: fewer NIF exports, simpler term handling, feature gates",
            size_kb, MAX_NIF_SIZE_KB
        );
    }

    #[test]
    fn test_size_report() {
        println!("\n=== 📊 AtomVM Size Report ===");
        
        // 📈 SIZE COMPARISON: Track size impact of different build configurations
        build_and_report("Base (no_std)", &["--no-default-features"]);
        build_and_report("NIFs (no_std + avmnif)", &["--features", "nifs"]);
        
        println!("=== End Size Report ===\n");
        println!("💡 Size Optimization Tips:");
        println!("   • Use #[cfg(feature = \"...\")] for optional functionality");
        println!("   • Prefer const generics over runtime polymorphism");
        println!("   • Minimize avmnif-rs term conversions");
        println!("   • Consider opt-level = \"z\" for size-critical builds");
    }

    // 🔧 SIZE HELPER: Build and measure different configurations
    fn build_and_report(name: &str, args: &[&str]) {
        let mut cmd_args = vec!["build", "--lib", "--release"];
        cmd_args.extend_from_slice(args);
        
        // 🎛️  SIZE LEVER: Could add additional optimization flags here
        // Example: .env("RUSTFLAGS", "-C opt-level=z -C lto=fat")
        let output = Command::new("cargo")
            .args(&cmd_args)
            .env("CARGO_CFG_NOT_TEST", "1")  // Ensure production build
            .output()
            .expect("Failed to build for report");
            
        if output.status.success() {
            let lib_path = "target/release/libavmgl_rs.rlib";
            if let Ok(metadata) = fs::metadata(lib_path) {
                let size_kb = metadata.len() as f64 / 1024.0;
                println!("{}: {:.2} KB", name, size_kb);
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════════════════════
    // 📝 SIZE OPTIMIZATION GUIDE:
    // ═══════════════════════════════════════════════════════════════════════════
    // 
    // 🎯 To reduce BASE library size:
    //   • Simplify hardware abstraction traits
    //   • Use const generics instead of dynamic dispatch  
    //   • Feature-gate debug/development code
    //   • Minimize use of alloc::vec, prefer arrays
    //   • Remove unused imports and dead code
    //
    // 🎯 To reduce NIF library size:
    //   • Minimize exported NIF functions
    //   • Simplify term conversion logic
    //   • Use lighter-weight alternatives to avmnif-rs if possible
    //   • Feature-gate advanced NIF functionality
    //   • Avoid unnecessary term allocations
    //
    // 🎯 Build-time optimizations:
    //   • RUSTFLAGS="-C opt-level=z" (optimize for size)
    //   • RUSTFLAGS="-C lto=fat" (link-time optimization)
    //   • cargo build --release (always for size measurement)
    //   • Strip symbols: RUSTFLAGS="-C strip=symbols"
    //
    // ═══════════════════════════════════════════════════════════════════════════
}