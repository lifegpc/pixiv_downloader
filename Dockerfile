FROM ubuntu:latest as builder
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
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /app
COPY . /app
ENV PREFIX=/app/clib
RUN cd ~ && git clone --depth 1 'https://github.com/Exiv2/exiv2' \
    && cd exiv2 && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=$PREFIX" \
    -DEXIV2_ENABLE_BROTLI=OFF -DEXIV2_ENABLE_INIH=OFF \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf exiv2
RUN cd ~ && git clone --depth 1 'https://github.com/nih-at/libzip' \
    && cd libzip && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=$PREFIX" \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf libzip
RUN cd ~ && git clone --depth 1 'https://code.videolan.org/videolan/x264.git' && cd x264 \
    && ./configure --disable-cli --enable-strip --enable-pic --enable-shared --disable-static --prefix=$PREFIX \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf x264
RUN cd ~ && git clone --depth 1 'https://git.ffmpeg.org/ffmpeg.git' && cd ffmpeg \
    && ./configure --enable-shared --disable-static --enable-gpl --enable-version3 --enable-libx264 --prefix=$PREFIX \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf ffmpeg
RUN export PKG_CONFIG_PATH=`pwd`/clib/lib/pkgconfig \
    && export CMAKE_PREFIX_PATH=`pwd`/clib \
    && export "LIBRARY_PATH=$LIBRARY_PATH:`pwd`/clib/lib" \
    && export "LD_LIBRARY_PATH=$LD_LIBRARY_PATH:`pwd`/clib/lib" \
    && cargo build --features all,docker --release

FROM ubuntu:latest as prod

WORKDIR /app

RUN apt update && apt install -y \
    zlib1g \
    libexpat1 \
    libssl3 \
    ca-certificates

COPY --from=builder /app/target/release/pixiv_downloader /app/pixiv_downloader
COPY --from=builder /app/clib/lib /app/lib
ENV LD_LIBRARY_PATH=/app/lib

ENTRYPOINT [ "/app/pixiv_downloader" ]
CMD [ "s" ]
