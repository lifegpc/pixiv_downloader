export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://git.ffmpeg.org/ffmpeg.git' && cd ffmpeg || exit 1
./configure "--prefix=${PREFIX}" --enable-shared --disable-static --enable-gpl --enable-version3 --disable-doc --enable-libx264 || exit 1
make -j8 && make install || exit 1
