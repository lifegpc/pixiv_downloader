export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://github.com/fmtlib/fmt' && cd fmt || exit 1
mkdir -p build && cd build || exit 1
cmake -DCMAKE_BUILD_TYPE=Release "-DCMAKE_INSTALL_PREFIX=$PREFIX" -DFMT_DOC=OFF -DFMT_TEST=OFF ../ || exit 1
make -j8 && make install || exit 1
