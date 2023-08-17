FROM debian:stable-slim

# Step 1: Installing dependencies
RUN apt-get update
RUN apt-get -y install bash binutils git gnupg2 xz-utils wget sudo

# Step 2: Installing Nix
RUN wget --output-document=/dev/stdout https://nixos.org/nix/install | sh -s -- --daemon
RUN . ~/.nix-profile/etc/profile.d/nix.sh
ENV PATH="/root/.nix-profile/bin:$PATH"

# Step 3: Installing devbox
RUN wget --quiet --output-document=/dev/stdout https://get.jetpack.io/devbox   | bash -s -- -f

# Step 4: Installing your devbox project
WORKDIR /code
COPY devbox.json devbox.json


RUN devbox run -- echo "Installed Packages."

RUN devbox shellenv --init-hook >> ~/.profile
