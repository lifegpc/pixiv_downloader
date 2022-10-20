from argparse import ArgumentParser
from hashlib import sha256 as _sha256
from os.path import exists
import sys
from time import gmtime, time, strftime
from typing import List


ALL_FEATURES = ['exiv2', 'expat', 'ffmpeg', 'libzip', 'openssl', 'pkgconf', 'x264', 'zlib']

def sha256(data) -> str:
    if isinstance(data, str):
        data = data.encode()
    elif not isinstance(data, bytes):
        data = str(data).encode()
    s = _sha256()
    s.update(data)
    return s.hexdigest()


def hash_file(feature, prefix) -> str:
    if prefix is None:
        fns = [f"build_{feature}.sh"]
    else:
        fns = []
        fns.append(f"build_{prefix}_{feature}.sh")
        fns.append(f"build_{prefix}_{feature}.bat")
        fns.append(f"download_{prefix}_{feature}.sh")
        fns.append(f"download_{prefix}_{feature}.bat")
    s = None
    for fn in fns:
        if exists(fn):
            if s is None:
                s = _sha256()
            with open(fn, 'rb') as f:
                c = f.read(256)
                while len(c) > 0:
                    s.update(c)
                    c = f.read(256)
    return s.hexdigest() if s is not None else ''


try:
    p = ArgumentParser(description='Get the cache key which used in action/cache')
    p.add_argument("features", help="The feature's name", action='append', nargs='+', choices=['all'] + ALL_FEATURES)
    p.add_argument('--prefix', help='The prefix of the cache key')
    args = p.parse_intermixed_args(sys.argv[1:])
    features: List[str] = args.features[0]
    if 'all' in features:
        features = ALL_FEATURES.copy()
    d = ''
    now = time()
    for i in features:
        dt = strftime('%Y-%m', gmtime(now))
        h = hash_file(i, args.prefix)
        d += f"{i}={dt}:{h}\n"
    print(d)
    print(f"::set-output name=cache_key::{sha256(d)}")
except Exception:
    from traceback import print_exc
    from sys import exit
    print_exc()
    exit(1)
