# Testing Strategy for avmgl-rs

## Overview

This document describes the testing approach for `avmgl-rs`, an OTM8009A display driver library designed for AtomVM integration. The library presents unique testing challenges due to its `no_std`/`no_main` embedded nature and optional NIF (Native Implemented Function) dependencies.

## Core Testing Philosophy

### Separation of Concerns
- **Hardware Driver Logic**: Testable without external dependencies
- **NIF Bindings**: Feature-gated and excluded from unit tests
- **Mock Hardware Interfaces**: Enable testing without actual hardware

### Test Environment vs Production Environment
- **Production**: `no_std` + `no_main` for embedded/AtomVM compatibility
- **Testing**: Standard Rust environment with `std` for test harness compatibility

## Key Lessons Learned

### 1. Conditional Compilation for no_std Libraries

**Problem**: `#![no_main]` attribute prevents test harness from generating required `main` function.

**Solution**: Use conditional compilation attributes:
```rust
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
```

**Why This Works**:
- Production builds get `no_std`/`no_main` for embedded compatibility
- Test builds use standard Rust runtime for test harness
- Tests don't run on target hardware anyway

### 2. Feature-Gated Dependencies

**Problem**: Optional NIF dependencies (`avmnif-rs`) cause compilation errors during testing.

**Solution**: Proper feature gating:
```rust
// In Cargo.toml
[dependencies]
avmnif-rs = { version = "0.3.0", optional = true }

[features]
default = ["nifs"]
nifs = ["avmnif-rs"]

// In source code
#[cfg(feature = "nifs")]
mod nif_bindings {
    use avmnif_rs::*;
    // NIF implementation
}
```

**Testing Command**:
```bash
cargo test --no-default-features
```

### 3. Library vs Binary Crate Configuration

**Correct `Cargo.toml` for Library**:
```toml
[lib]
name = "avmgl_rs"
crate-type = ["lib"]
```

**Not**:
```toml
crate-type = ["cdylib"]  # This is for dynamic libraries
```

## Testing Architecture

### Directory Structure
```
src/
├── lib.rs                    # Main library with conditional compilation
├── otm8009a/
│   ├── driver.rs            # Core hardware driver (always testable)
│   ├── nifs.rs              # Feature-gated NIF bindings
│   └── mod.rs               # Module organization
├── traits.rs               # Hardware abstraction traits
└── testing/
    ├── mod.rs               # Test module organization
    ├── mocks.rs             # Mock hardware implementations
    ├── nifs.rs              # NIF logic tests (without actual NIFs)
    └── traits.rs            # Trait testing utilities
```

### Mock Strategy

**Hardware Abstraction**: All hardware interactions go through traits:
```rust
pub trait DsiInterface {
    fn send_dcs_command(&mut self, len: usize, data: &[u8]) -> Result<(), DsiError>;
    fn delay_ms(&mut self, ms: u32);
    fn is_ready(&self) -> bool;
    fn reset(&mut self) -> Result<(), DsiError>;
}
```

**Mock Implementations**: Enable testing without hardware:
```rust
pub struct MockDsiInterface {
    pub commands_sent: Vec<(usize, Vec<u8>)>,
    pub delays: Vec<u32>,
    pub ready_state: bool,
}
```

### Test Categories

#### 1. Unit Tests (Core Driver Logic)
- Driver state management
- Parameter validation
- Display initialization sequences
- Coordinate boundary checking
- Color format conversions

#### 2. Integration Tests (Mock Hardware)
- Full driver workflow with mock interfaces
- Error propagation through trait boundaries
- Display configuration scenarios

#### 3. NIF Logic Tests (Without AtomVM)
- Term parsing and validation logic
- RGB color conversion functions
- Parameter boundary conditions
- Error handling without actual NIF runtime

## Command Reference

### Development Workflow
```bash
# Check compilation without NIFs
cargo check --lib --no-default-features

# Run all tests (no NIF dependencies)
cargo test --no-default-features

# Run specific test module
cargo test --no-default-features testing::nifs

# Run tests with output
cargo test --no-default-features -- --nocapture

# Build production library (with NIFs)
cargo build --lib --release
```

### CI/CD Considerations
```bash
# Test matrix should include:
cargo test --no-default-features        # Core functionality
cargo check --lib                       # Full build with NIFs
cargo build --lib --release             # Production build
```

## Benefits of This Approach

### ✅ What Works Well
- **Complete separation** of hardware logic from NIF bindings
- **Fast test cycles** without external dependencies
- **LSP compatibility** during development
- **Comprehensive coverage** of driver logic
- **Production-ready** NIF integration when needed

### ✅ Testable Components
- Display driver state machine
- Parameter validation logic
- Hardware initialization sequences
- Error handling and propagation
- Color space conversions
- Coordinate transformations

### ✅ Mock Verification
- Command sequences sent to hardware
- Timing and delay requirements
- Error condition handling
- State transitions

## Future Improvements

### Potential Enhancements
- Property-based testing for coordinate validation
- Fuzzing for RGB color conversion functions
- Performance benchmarks for hot paths
- Integration tests with real hardware (optional)

### Documentation Tests
```rust
/// # Example
/// ```
/// # use avmgl_rs::*;
/// # use avmgl_rs::testing::mocks::*;
/// let mut driver = OTM8009ADriver::new(
///     MockDsiInterface::new(),
///     MockLtdcInterface::new(), 
///     MockFramebufferInterface::new()
/// );
/// driver.init(OTM8009A_FORMAT_RGB565, OTM8009A_ORIENTATION_LANDSCAPE)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
```

## Troubleshooting

### Common Issues

**"undefined symbol `_main`"**
- Ensure `#![cfg_attr(not(test), no_main)]` is used
- Run tests with `--no-default-features`

**"unresolved import `avmnif_rs`"**
- Check feature gates: `#[cfg(feature = "nifs")]`
- Use `--no-default-features` flag

**"unused import warnings"**
- Normal during testing when features are disabled
- Can be fixed with `cargo fix --lib --tests`

### Verification Commands
```bash
# Verify conditional compilation works
cargo check --lib --no-default-features  # Should succeed
cargo check --lib                        # Should succeed with NIFs

# Verify tests run clean
cargo test --no-default-features         # Should show passing tests
```

## Conclusion

This testing strategy successfully balances the constraints of embedded `no_std` development with the need for comprehensive testing. By using conditional compilation and feature gates, we achieve:

- **Development agility** with fast test cycles
- **Production reliability** with proper embedded constraints  
- **Maintainability** through clear separation of concerns
- **Portability** across different target environments

The approach serves as a template for other embedded Rust libraries that need to integrate with specialized runtimes like AtomVM while maintaining testability.