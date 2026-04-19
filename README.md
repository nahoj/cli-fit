**inband** lets you run an interactive command while constraining its output to a specified number of lines of the terminal. A non-invasive alternative to multiplexers like `tmux` and `screen` that take control of the whole screen. inband does not work with such programs that use the alt screen, including text editors (micro, nano), pagers (less), etc.

```shell
$ band 5 tail -f /var/log/seqlog
7
8
9
10
^C
$ band 5 mpv scherzo.wma # ;)
 Genre: Classical
 Title: Symphony No. 9 (Scherzo)
 Track: 1
AO: [pipewire] 44100Hz stereo 2ch floatp
A: 00:00:42 / 00:01:15 (56%)
```

## Install

Developed on Linux, untested on other systems.

```shell
cargo install --git https://github.com/nahoj/inband
```

```shell
mise use -g 'cargo:https://github.com/nahoj/inband@branch:main'
```

From within the repo:

```shell
cargo install --path .
```
