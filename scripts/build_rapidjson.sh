export PREFIX=`pwd`/clib
mkdir -p cbuild && cd cbuild || exit 1
git clone --depth 1 'https://github.com/Tencent/rapidjson' && cd rapidjson || exit 1
mkdir -p build && cd build || exit 1
cmake -DCMAKE_BUILD_TYPE=Release .. "-DCMAKE_INSTALL_PREFIX=$PREFIX" -DRAPIDJSON_BUILD_DOC=OFF -DRAPIDJSON_BUILD_EXAMPLES=OFF -DRAPIDJSON_BUILD_TESTS=OFF || exit 1
make -j8 && make install || exit 1
