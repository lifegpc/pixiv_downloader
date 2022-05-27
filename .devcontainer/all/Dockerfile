FROM ubuntu:latest
RUN apt update && apt install -y \
    gcc \
    'g++' \
    cmake \
    nasm \
    git \
    zlib1g-dev \
    libexpat1-dev \
    pkgconf \
    clang \
    autoconf \
    automake \
    autotools-dev \
    libtool \
    xutils-dev \
    libssl-dev \
    ca-certificates \
    curl \
    file
RUN cd ~ && git clone --depth 1 'https://github.com/Exiv2/exiv2' \
    && cd exiv2 && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=/usr" \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf exiv2
RUN cd ~ && git clone --depth 1 'https://github.com/nih-at/libzip' \
    && cd libzip && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=/usr" \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf libzip
RUN cd ~ && git clone --depth 1 'https://code.videolan.org/videolan/x264.git' && cd x264 \
    && ./configure --disable-cli --enable-strip --enable-pic --enable-shared --disable-static --prefix=/usr \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf x264
RUN cd ~ && git clone --depth 1 'https://git.ffmpeg.org/ffmpeg.git' && cd ffmpeg \
    && ./configure --enable-shared --disable-static --enable-gpl --enable-version3 --enable-libx264 --prefix=/usr \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf ffmpeg
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y
ENV PATH=/root/.cargo/bin:$PATH
# Enable GPG sign by default
RUN git config commit.gpgsign true
