#!/bin/bash
set -e
(echo 'wait for pipewire') || exit 1
(pw-cli ls 0 | grep --quiet 'id 0, type PipeWire:Interface:Core/4') || exit 1