from argparse import ArgumentParser
from hashlib import sha256 as _sha256
import sys
from time import gmtime, time, strftime
from typing import List


ALL_FEATURES = ['exiv2']

def sha256(data) -> str:
    if isinstance(data, str):
        data = data.encode()
    elif not isinstance(data, bytes):
        data = str(data).encode()
    s = _sha256()
    s.update(data)
    return s.hexdigest()


try:
    p = ArgumentParser(description='Get the cache key which used in action/cache')
    p.add_argument("features", help="The feature's name", action='append', nargs='+', choices=['all'] + ALL_FEATURES)
    args = p.parse_intermixed_args(sys.argv[1:])
    features: List[str] = args.features
    if 'all' in features:
        features = ALL_FEATURES.copy()
    d = ''
    now = time()
    for i in features:
        dt = strftime('%Y-%m', gmtime(now))
        d += f"i={dt}"
    print(f"::set-output name=cache_key::{sha256(d)}")
except Exception:
    from traceback import print_exc
    from sys import exit
    print_exc()
    exit(1)
