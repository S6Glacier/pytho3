{
  inputs.nru.url = "github:voidcontext/nix-rust-utils/v0.4.1";

  outputs = { nru, ...}: 
    nru.lib.mkOutputs ({...}: {
      src = ./.;
      pname = "indieweb-tools";
      version = "0.1.0";
    });
}
