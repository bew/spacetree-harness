{
  lib,
  rustPlatform,
  bats,
  jq,
  makeWrapper,
}:

# Ships the spacetree Rust binary; jq is only needed to pretty-print on a terminal.
rustPlatform.buildRustPackage {
  pname = "spacetree";
  version = "0.1.0";

  # Only what the crate build and the bats suite need.
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./spacetree.bats
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

  # The installed binary is what gets tested, so the suite runs after installPhase.
  doCheck = false;

  postInstall = ''
    # Point the test file at the binary this derivation just installed
    substituteInPlace ./spacetree.bats \
      --replace-fail 'SCRIPT_PATH="$SCRIPT_DIR/spacetree"' SCRIPT_PATH="$out/bin/spacetree"
    # The suite parses the tool's JSON, so jq has to be reachable on PATH
    PATH=${lib.makeBinPath [ jq ]}:$PATH ${lib.getExe bats} spacetree.bats
  '';

  meta = {
    description = "Spacetree CLI: resolve and inspect the enclosing spacetree root";
    mainProgram = "spacetree";
    platforms = lib.platforms.unix;
  };
}
