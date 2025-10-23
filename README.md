# Open IP radio

A IP radio station and client built in rust.

## Dependences

You will need to install these dependencies:

Ubuntu/Debian:
```bash
sudo apt-get install libasound2-dev
```

Fedora:
```bash
sudo dnf install alsa-lib-devel
```

Arch Linux:
```bash
sudo pacman -S alsa-lib
```

## Usage

To run the server:
```bash
make station
```

To run the client:
```bash
make client
```
