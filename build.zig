const std = @import("std");

pub fn build(b: *std.Build) void {
    // const target = b.standardTargetOptions(.{});
    // const optimize = b.standardOptimizeOption(.{});

    // Step 1: Generate Qt resources automatically
    const rcc_cmd = b.addSystemCommand(&.{ "rcc", "ui/resources.qrc", "-o", "qrc_resources.cpp" });

    // Create the executable
    const mod = b.createModule(.{
        .target = b.graph.host,
    });

    mod.addCSourceFiles(.{
        .files = &.{
            "main.cpp",
            "qrc_resources.cpp",
        },
        .flags = &.{
            "-std=c++20",
            "-Wall",
            "-Wextra",
        },
    });

    const exe = b.addExecutable(.{
        .name = "waycast",
        .root_module = mod,
    });

    exe.step.dependOn(&rcc_cmd.step);

    // Link C++ standard library
    exe.linkLibCpp();
    // Link Qt6
    exe.linkSystemLibrary("Qt6Core");
    exe.linkSystemLibrary("Qt6Quick");
    exe.linkSystemLibrary("Qt6Gui");
    exe.linkSystemLibrary("Qt6Qml");

    // Install the executable
    b.installArtifact(exe);

    // Create a run step
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());

    // Allow passing arguments to the program
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    // Create the run step
    const run_step = b.step("run", "Run the application");
    run_step.dependOn(&run_cmd.step);
}
