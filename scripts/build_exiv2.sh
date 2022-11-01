export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://github.com/Exiv2/exiv2' && cd exiv2 || exit 1
mkdir -p build && cd build || exit 1
cmake -DCMAKE_BUILD_TYPE=Release "-DCMAKE_INSTALL_PREFIX=$PREFIX" -DEXIV2_ENABLE_BROTLI=OFF ../ || exit 1
make -j8 && make install || exit 1
