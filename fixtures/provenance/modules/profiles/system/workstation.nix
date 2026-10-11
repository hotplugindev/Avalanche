{ config, lib, ... }:
{
  services.openssh.enable = lib.mkDefault true;
  services.xserver.enable = lib.mkIf config.gb.host.hardware.hasBattery true;
  gb.requires.system.audio.pipewire = [ "profiles.system.workstation" ];
}
