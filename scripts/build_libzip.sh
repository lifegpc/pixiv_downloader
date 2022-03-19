export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://github.com/nih-at/libzip' && cd libzip || exit 1
mkdir -p build && cd build || exit 1
cmake -DCMAKE_BUILD_TYPE=Release "-DCMAKE_INSTALL_PREFIX=$PREFIX" -DBUILD_TOOLS=OFF -DBUILD_REGRESS=OFF -DBUILD_EXAMPLES=OFF -DBUILD_DOC=OFF ../ || exit 1
make -j8 && make install || exit 1
