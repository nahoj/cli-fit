**cli-fit** lets you run an interactive command while constraining its output to a specified number of lines of the terminal. A non-invasive alternative to full-screen multiplexers like tmux and screen.

```shell
$ fit 5 tail -f /var/log/seqlog
7
8
9
10
^C
$ fit 5 mpv scherzo.wma # ;)
 Genre: Classical
 Title: Symphony No. 9 (Scherzo)
 Track: 1
AO: [pipewire] 44100Hz stereo 2ch floatp
A: 00:00:42 / 00:01:15 (56%)
```

You can even run a shell with it:

```shell
xonsh @ fit 5 zsh
zsh % seq 1 10 # <Enter>
```

becomes:

```shell
xonsh @ fit 5 zsh
7
8
9
10
zsh % 
```

cli-fit will not behave as intended if used to run:
- programs that use the alt screen, such as multiplexers (tmux), text editors (micro), pagers (less), etc.
- itself.

However, cli-fit works fine *inside* tmux and screen.

## Install

Developed on Linux, untested on other systems. Use one of:

```shell
cargo install --git https://github.com/nahoj/cli-fit
```

```shell
mise use -g 'cargo:https://github.com/nahoj/cli-fit@branch:main'
```

From within the repo:

```shell
cargo install --path .
```
