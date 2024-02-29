# Build the docker file using `docker build . -t "flux"`
# Then copy the file using `docker cp CONTAINER_ID:/app/flux-screensavers/result/bin/flux-screensaver-setup-*.exe ./`
FROM nixos/nix

WORKDIR /app
RUN git clone https://github.com/sandydoo/flux-screensavers.git
WORKDIR /app/flux-screensavers
RUN nix build .#windows.installer --extra-experimental-features "flakes nix-command"
#RUN nix build .#windows.flux

CMD ["tail", "-f", "/dev/null"]