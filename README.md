# apt-updates-notifier

Checks for package updates and sends a notification email.

# Install

```
sudo dpkg -i apt-updates-notifier.deb

# Configure before enabling
sudo systemctl enable apt-updates-notifier.service apt-updates-notifier.timer
```

# Config

Edit `/etc/pkg-updates-notifier.config`.

To change the frequency of checks, edit the timer.

```
sudo systemctl edit apt-updates-notifier.timer
```

# Development

## Debian Package

`libapt-pkg-dev` must be installed.

Install

```
cargo install cargo-deb
```

Build package.

```
cargo deb
```