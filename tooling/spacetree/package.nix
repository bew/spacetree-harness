{
  lib,
  rustPlatform,
  jq,
  makeWrapper,
}:

# Ships the spacetree Rust binary; jq is only needed to pretty-print on a terminal.
rustPlatform.buildRustPackage {
  pname = "spacetree";
  version = "0.1.0";

  # Only what the crate build and its tests need.
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./tests
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    makeWrapper
  ];

  # jq is optional at runtime — the tool falls back to plain JSON without it —
  # but putting it on PATH is what gives terminal users jq's colored output.
  postFixup = ''
    wrapProgram $out/bin/spacetree --prefix PATH : ${lib.makeBinPath [ jq ]}
  '';

  meta = {
    description = "Spacetree CLI: resolve and inspect the enclosing spacetree root";
    mainProgram = "spacetree";
    platforms = lib.platforms.unix;
  };
}
