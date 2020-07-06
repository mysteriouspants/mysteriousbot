#!/bin/sh

cargo build --release

if [ $? -eq 0 ]; then
  ssh xpm@mysteriouspants.com sudo systemctl stop cmdr
  scp config/cmdr.toml cmdr@mysteriouspants.com:/home/cmdr/config/cmdr.toml
  scp target/release/cmdr cmdr@mysteriouspants.com:/home/cmdr/cmdr
  ssh cmdr@mysteriouspants.com chmod +x /home/cmdr/cmdr
  ssh xpm@mysteriouspants.com sudo systemctl start cmdr
else
  echo "Fix your broken build, man."
fi
