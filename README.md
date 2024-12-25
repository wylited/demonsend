# Demonsend

A daemon and command line client utility for [localsend](https://localsend.org), powered by [localsend-rs](https://github.com/wylited/localsend).

Supports IPC through the unix socket at `/tmp/demonsend.sock`

## Install

Demonsend is currently only tested on linux and **requires** a dialog system like kdialog, Zenity, Glade and Yad.

To install, the preferred method is cargo.

``` shell
cargo install demonsend
```

Or you can compile it locally.

``` shell
git clone https://github.com/wylited/demonsend
cd demonsend
cargo build --release
```

Your binary will be located in `demonsend/target/release/`

## Usage

To start, configure the daemon interactively

``` shell
demonsend config init
```

and then you can start your daemon.

``` shell
demonsend start
```

Once you open demonsend on your phone, you will be able to see demonsend as a client. Upon receiving a file, demonsend will prompt a notification on your laptop.

You can send a file by first listing the available peers and sending a file to one of them!

``` shell
demonsend peers
demonsend file <peer_fingerprint> ~/Pictures/tokyonight.jpg
```

## Issues

If you run into any issues, open a github issue or dm me on the hackclub slack or discord `@wylited`
