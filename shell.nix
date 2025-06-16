# shell.nix
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Build-Tools
    pkg-config
    cmake
    gnumake # Stellt das 'make' Kommando bereit

    # System-Bibliotheken, die von Rust-Crates benötigt werden
    udev.dev         # Für libudev-sys
    wayland.dev      # Für wayland-sys (wayland-server)
    freetype.dev     # Für freetype-sys
    systemd.dev      # Für sd-sys (libsystemd)
    expat.dev        # Für expat-sys
    glib.dev         # Für glib-sys, gobject-sys, gio-sys (allgemeine GLib-Entwicklungspakete)
    cairo.dev        # Für cairo-sys-rs
    pango.dev        # Für pango-sys
    gdk-pixbuf.dev   # Für gdk-pixbuf-sys
    gtk3.dev # Stellt gdk-3.0 bereit


    # Rust Toolchain (optional, aber empfohlen für konsistente Builds)
    rustc
    cargo
  ];

  # Setze PKG_CONFIG_PATH, um sicherzustellen, dass pkg-config die .pc-Dateien findet.
  # mkShell setzt dies normalerweise automatisch, aber explizit schadet nicht.
  # PKG_CONFIG_PATH = "${lib.makeSearchPath "pkgconfig" (filter (p: p ? "/lib/pkgconfig") buildInputs)}";

  # Optionale Umgebungsvariablen für Rust-Builds
  # export NIX_LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
