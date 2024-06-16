#!/bin/bash

set -e

cd ..
tar -czf tailord/tailord.tar.gz \
  LICENSE \
  Cargo.toml \
  Cargo.lock \
  tailor_api/ \
  tailor_client/ \
  tailor_cli/ \
  tailor_hwcaps/ \
  tuxedo_ioctl/ \
  tuxedo_sysfs/ \
  tailord/Cargo.toml \
  tailord/src/ \
  tailord/meson.build \
  tailord/meson_options.txt \
  tailord/post_install.sh \
  tailord/tailord.service.in \
  tailord/tailord.spec \
  tailord/com.tux.Tailor.conf \
  tailord/CHANGELOG.md

cd tailord/
rpkg srpm --outdir .
