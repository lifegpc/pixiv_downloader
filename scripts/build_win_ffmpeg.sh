export PREFIX=`pwd`/clib
export PREFIX2=`cygpath -w $PREFIX`
export "PATH=$PREFIX/bin:$PATH"
export PKG_CONFIG_PATH=$PREFIX/lib/pkgconfig
export "LIB=$LIB;$PREFIX2/lib"
export "INCLUDE=$INCLUDE;$PREFIX2/include"
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://github.com/FFmpeg/FFmpeg' ffmpeg && cd ffmpeg || exit 1
./configure \
    --enable-gpl \
    --enable-shared \
    --disable-static \
    --enable-version3 \
    --prefix=${PREFIX2//\\//} \
    --disable-doc \
    --enable-libx264 \
    --disable-programs \
    --pkg-config-flags="--env-only" \
    --toolchain=msvc
if [ $? != 0 ]; then
    cat ffbuild/config.log
    exit 1
fi
make -j8 || exit 1
make -j8 install || exit 1
mv -fv $PREFIX/bin/*.lib $PREFIX/lib || exit 1
mv -fv */*.pdb $PREFIX/bin || exit 1
