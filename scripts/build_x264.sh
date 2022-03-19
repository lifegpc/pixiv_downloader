export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://code.videolan.org/videolan/x264.git' && cd x264 || exit 1
./configure "--prefix=${PREFIX}" --disable-cli --enable-strip --enable-pic || exit 1
make -j8 && make install || exit 1
