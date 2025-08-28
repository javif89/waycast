const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Create the executable
    const exe = b.addExecutable(.{
        .name = "waycast",
        .target = target,
        .optimize = optimize,
    });

    // Set C++20 standard
    exe.linkLibCpp();

    // Add include directories (equivalent to CMake include_directories)
    exe.addIncludePath(b.path("lib"));
    exe.addIncludePath(b.path("lib/ui"));
    exe.addIncludePath(b.path("lib/plugins"));
    exe.addIncludePath(b.path("src"));

    // Add source files (equivalent to CMake GLOB_RECURSE and target_sources)
    const src_files = [_][]const u8{
        // Main source
        "src/main.cpp",
        
        // Library UI files
        "lib/ui/AppListModel.cpp",
        "lib/ui/ListItem.cpp",
        
        // Plugin files
        "src/plugins/DesktopAppPlugin/DesktopAppListItem.cpp",
    };

    exe.addCSourceFiles(.{
        .files = &src_files,
        .flags = &[_][]const u8{
            "-std=c++20",
            "-fmodules-ts",
            "-mavx2",
            "-Wall",
            "-Wextra",
        },
    });

    // System library dependencies (equivalent to find_package and target_link_libraries)
    // Qt6 libraries
    exe.linkSystemLibrary("Qt6Core");
    exe.linkSystemLibrary("Qt6Gui");
    exe.linkSystemLibrary("Qt6Qml");
    exe.linkSystemLibrary("Qt6Quick");
    exe.linkSystemLibrary("Qt6QuickControls2");
    exe.linkSystemLibrary("Qt6Widgets");
    
    // LayerShellQt
    exe.linkSystemLibrary("LayerShellQtInterface");
    
    // GIO (GLib) - equivalent to pkg_check_modules(GIO REQUIRED gio-2.0)
    exe.linkSystemLibrary("gio-2.0");
    exe.linkSystemLibrary("gobject-2.0");
    exe.linkSystemLibrary("glib-2.0");

    // Add pkg-config paths for system libraries
    exe.addLibraryPath(.{ .cwd_relative = "/usr/lib" });
    exe.addLibraryPath(.{ .cwd_relative = "/usr/local/lib" });

    // QML resource compilation (equivalent to qt_add_qml_module)
    // Note: This is simplified - full Qt QML module support would need more complex setup
    _ = b.addSystemCommand(&.{
        "rcc", "-binary", "-o", "qml_resources.rcc", "ui/Main.qml"
    });
    
    // Set RPATH for runtime library discovery (equivalent to INSTALL_RPATH)
    if (target.result.os.tag == .linux) {
        exe.addRPath(.{ .cwd_relative = "$ORIGIN" });
    }

    // Install the executable
    b.installArtifact(exe);

    // Create run command
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    // Allow passing arguments
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    // Create run step
    const run_step = b.step("run", "Run waycast");
    run_step.dependOn(&run_cmd.step);
    
    // Create build step (equivalent to make bld)
    const build_step = b.step("bld", "Build waycast");
    build_step.dependOn(&exe.step);
    
    // Combined build and run step (equivalent to make br)
    const build_run_step = b.step("br", "Build and run waycast");
    build_run_step.dependOn(&run_cmd.step);
}