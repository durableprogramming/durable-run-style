{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [ git libyaml openssl openssl.dev];

  languages.rust.enable = true;


  enterShell = ''

  '';

}
