# Create a library target that links against the pre-built Rust static library
add_library(avmgl_rs STATIC IMPORTED)
set_target_properties(avmgl_rs PROPERTIES
    IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/target/release/libavmgl_rs.a
)

# Create the main library target
add_library(avmgl INTERFACE)
target_link_libraries(avmgl INTERFACE avmgl_rs)

# Use whole-archive to ensure all NIF symbols are included
target_link_options(avmgl INTERFACE -Wl,--whole-archive ${CMAKE_CURRENT_SOURCE_DIR}/target/release/libavmgl_rs.a -Wl,--no-whole-archive)