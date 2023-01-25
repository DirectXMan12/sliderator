#!/usr/bin/env bash
# Copyright 2021 Solly Ross

set -e

src=$(realpath ${1:?must specify a markdown source file})
template=$(realpath ${2:?must specify a template HTML file})

src_file=$(basename ${src})
src_dir=$(dirname ${src})

run_path=$(dirname ${BASH_SOURCE[0]})/run.sh

set +e
${run_path} ${src} ${template} > ${src_dir}/${src_file}.out.html
set -e

inotifywait -e close_write,moved_to,create -m ${src_dir} |
while read -r directory events filename; do
    if [ "${filename}" = "${src_file}" ]; then
        set +e # so we retry the loop on failure
        ${run_path} ${src} ${template} > ${src_dir}/${src_file}.out.html
        set -e
    fi
done
