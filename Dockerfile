FROM ubuntu:devel as builder
RUN apt-get update && apt-get install -y \
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
    file \
    gettext \
    python3 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain nightly -y
ENV PATH=/root/.cargo/bin:$PATH
RUN cd ~ && git clone --depth 1 'https://github.com/Exiv2/exiv2' \
    && cd exiv2 && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=/clib" \
    -DEXIV2_ENABLE_BROTLI=OFF -DEXIV2_ENABLE_INIH=OFF \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf exiv2
RUN cd ~ && git clone --depth 1 'https://github.com/nih-at/libzip' \
    && cd libzip && mkdir -p build && cd build \
    && cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=/clib" \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf libzip
RUN cd ~ && git clone --depth 1 'https://code.videolan.org/videolan/x264.git' && cd x264 \
    && ./configure --disable-cli --enable-strip --enable-pic --enable-shared --disable-static --prefix=/clib \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf x264
RUN export PKG_CONFIG_PATH=/clib/lib/pkgconfig \
    && cd ~ && git clone --depth 1 'https://git.ffmpeg.org/ffmpeg.git' && cd ffmpeg \
    && ./configure --enable-shared --disable-static --enable-gpl --enable-version3 --enable-libx264 --prefix=/clib \
    && make -j$(grep -c ^processor /proc/cpuinfo) && make install \
    && cd ~ && rm -rf ffmpeg
RUN cd ~ && \
    curl -L "https://github.com/curl/curl/releases/download/curl-8_8_0/curl-8.8.0.tar.gz" -o curl-8.8.0.tar.gz && \
    tar -xzvf curl-8.8.0.tar.gz && \
    cd curl-8.8.0 && \
    mkdir build && cd build && \
    cmake -DCMAKE_BUILD_TYPE=Release -DCURL_DISABLE_ALTSVC=ON -DCURL_DISABLE_SRP=ON \
    -DCURL_DISABLE_COOKIES=ON -DCURL_DISABLE_BASIC_AUTH=ON -DCURL_DISABLE_BEARER_AUTH=ON \
    -DCURL_DISABLE_DIGEST_AUTH=ON -DCURL_DISABLE_KERBEROS_AUTH=ON -DCURL_DISABLE_NEGOTIATE_AUTH=ON \
    -DCURL_DISABLE_AWS=ON -DCURL_DISABLE_DICT=ON -DCURL_DISABLE_DOH=ON -DCURL_DISABLE_FILE=ON \
    -DCURL_DISABLE_FORM_API=ON -DCURL_DISABLE_FTP=ON -DCURL_DISABLE_GETOPTIONS=ON \
    -DCURL_DISABLE_GOPHER=ON -DCURL_DISABLE_HEADERS_API=ON -DCURL_DISABLE_HSTS=ON \
    -DCURL_DISABLE_HTTP_AUTH=ON -DCURL_DISABLE_IMAP=ON -DCURL_DISABLE_LDAP=ON \
    -DCURL_DISABLE_LDAPS=ON -DCURL_DISABLE_LIBCURL_OPTION=ON -DCURL_DISABLE_MQTT=ON \
    -DCURL_DISABLE_NETRC=ON -DCURL_DISABLE_NTLM=ON -DCURL_DISABLE_POP3=ON \
    -DCURL_DISABLE_PROXY=ON -DCURL_DISABLE_RTSP=ON -DCURL_DISABLE_SMB=ON \
    -DCURL_DISABLE_SMTP=ON -DCURL_DISABLE_TELNET=ON -DCURL_DISABLE_TFTP=ON \
    -DUSE_MANUAL=OFF -DCURL_ENABLE_SSL=OFF -DUSE_LIBIDN2=ON -DCURL_USE_LIBPSL=OFF \
    -DCURL_USE_LIBSSH2=OFF -DCMAKE_INSTALL_PREFIX=/clib -DBUILD_TESTING=OFF ../ && \
    make -j$(grep -c ^processor /proc/cpuinfo) && make install && \
    cd ~ && rm -rf curl-8.8.0 curl-8.8.0.tar.gz
WORKDIR /app
COPY . /app
RUN export PKG_CONFIG_PATH=/clib/lib/pkgconfig \
    && export CMAKE_PREFIX_PATH=/clib \
    && export "LIBRARY_PATH=$LIBRARY_PATH:/clib/lib" \
    && export "LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/clib/lib" \
    && cargo build --features all,docker --release
RUN python3 scripts/gen_mo.py -o /app/i18n-output

FROM ubuntu:devel as prod

WORKDIR /app

RUN apt-get update && apt-get install -y \
    zlib1g \
    libexpat1 \
    libssl3 \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pixiv_downloader /app/pixiv_downloader
COPY --from=builder /clib/lib /app/lib
COPY --from=builder /app/i18n-output /app
COPY --from=builder /clib/bin /app/bin
ENV LD_LIBRARY_PATH=/app/lib
ENV PATH=/app/bin:$PATH
ENV LC_ALL=C.utf8

RUN mkdir -p /app/data && mkdir -p /app/downloads && mkdir -p /app/temp

ENTRYPOINT [ "/app/pixiv_downloader" ]
CMD [ "s" ]

HEALTHCHECK --interval=30s --timeout=30s --start-period=10s --retries=3 \
    CMD curl -Lk -fsS http://localhost:8080/api/version || exit 1
