export PREFIX=`pwd`/clib
export PREFIX2=`cygpath -w $PREFIX`
export "PATH=$PREFIX/bin:$PATH"
export PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig
export "LIB=$LIB;$PREFIX2/lib"
export "INCLUDE=$INCLUDE;$PREFIX2/include"
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 "https://code.videolan.org/videolan/x264.git" && cd x264 || exit 1
export CC=cl
export PKGCONFIG='pkg-config --msvc --env-only'
./configure \
    --prefix=${PREFIX2//\\//} \
    --disable-cli \
    --enable-shared \
    || exit 1
make -j8 || exit 1
make -j8 install || exit 1
mv -v $PREFIX/lib/libx264.dll.lib $PREFIX/lib/libx264.lib || exit 1
