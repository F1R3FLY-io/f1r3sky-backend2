FROM ghcr.io/f1r3fly-io/rnode:latest

COPY --chown=daemon genesis /var/lib/rnode/genesis
