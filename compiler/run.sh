#!/usr/bin/env bash
# Copyright 2021 Solly Ross

set -e

src=$(realpath ${1:?must specify a markdown source file})
template=$(realpath ${2:?must specify a template HTML file})

cd $(dirname ${BASH_SOURCE[0]})
/opt/bin/pandoc -t json -f markdown-citations ${src} | cargo run --release ${template}
