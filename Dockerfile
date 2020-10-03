FROM ubuntu:latest

RUN apt-get update && apt-get install -y \
    curl \
    git \
    locales \
    python \
    zsh

RUN locale-gen en_US.UTF-8
ENV LANG en_US.UTF-8
ENV LC_ALL en_US.UTF-8

RUN curl --proto '=https' -fLsS https://rossmacarthur.github.io/install/crate.sh \
    | bash -s -- --repo "sharkdp/hyperfine" --to /usr/local/bin

# Antibody
RUN curl -fLsS git.io/antibody | sh -s - -b /usr/local/bin

# Antigen
RUN curl -fLsS -o /root/antigen.zsh https://git.io/antigen

# Sheldon
RUN curl --proto '=https' -fLsS https://rossmacarthur.github.io/install/crate.sh \
    | bash -s -- --repo "rossmacarthur/sheldon" --tag 0.5.3 --to /usr/local/bin

# Zgen
RUN git clone https://github.com/tarjoilija/zgen.git /root/.zgen

# Zinit
RUN mkdir -p /root/.zinit \
    && git clone https://github.com/zdharma/zinit.git /root/.zinit/bin

# Zplug
RUN git clone https://github.com/zplug/zplug /root/.zplug
