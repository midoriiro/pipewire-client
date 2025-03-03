#!/bin/bash
mkdir --parents ${PIPEWIRE_RUNTIME_DIR}
supervisord -c /root/supervisor.conf
rm --force --recursive ${PIPEWIRE_RUNTIME_DIR}