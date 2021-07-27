# Gelato
<div align="center">
  <img src="https://github.com/lilybrevec/gelato/blob/images/gelato-logo.png" alt="gelato" title="Gelato Logo">
  <p> A GUI IRC Client. Be Pop and Cool! </p>
</div>

## How 2 Use
### MacOS

```sh
$ brew install openssl
$ cp config.toml.template config.toml
$ emacs config.toml # edit infomation
$ cargo run
```

### Ubuntu
- Environment
  - Ubuntu 21.04 (Hirsute Hippo)
  - CPU:INTEL Core i7-9700KF 3.6GHz 12MB 8cores/8threads
  - GPU:MSI GEFORCE RTX


```sh
$ sudo apt update && sudo apt upgrade
$ sudo apt install gcc pkg-config openssl libasound2-dev cmake build-essential python3 libfreetype6-dev libexpat-dev libexpat1-dev libxcb-composite0-dev libssl-dev libx11-dev gcc-multilib libxkbcommon-dev libfontconfig1-dev

# If you see "vulkan: No DRI3 support detected" message, you can try the below.

$ cat /etc/X11/xorg.conf.d/20-intel.conf
Section "Device"
   Identifier  "Intel Graphics"
   Driver      "intel"
   Option      "DRI"     "3"
EndSection
$ reboot

$ cp config.toml.template config.toml
$ emacs config.toml # edit infomation
$ cargo run
```

### Attention
- At 2021/07/29 this app is underconstruction.
- You should set config.toml and set hostname that can connect.

## Release
- [Ver. 0.0.4](https://github.com/lilybrevec/gelato/releases/tag/0.0.4)

## Look
<div align="center">
   <img src="https://github.com/lilybrevec/gelato/blob/images/gelato-screen.png" alt="gelato" title="Gelato Screen">
</div>