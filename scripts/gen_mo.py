from argparse import ArgumentParser
from os.path import abspath, dirname, exists,  join
from os import listdir, makedirs
from sys import exit as _exit
from traceback import print_exc
from subprocess import Popen
from re import compile


def call(*cmd):
    print(cmd)
    p = Popen(cmd)
    p.wait()
    return p.returncode


try:
    RE = compile(r'^pixiv_downloader\.(.*).po$')
    p = ArgumentParser()
    p.add_argument('-p', '--prefix', help='Translation will install at PREFIX/share/locale/<locale>/LC_MESSAGES/pixiv_downloader.mo')  # noqa: E501
    p.add_argument('-o', '--output', help='Output all translations file in a directory.')  # noqa: E501
    d = abspath(join(dirname(__file__), "../Language"))
    arg = p.parse_intermixed_args()
    if arg.prefix is None and arg.output is None:
        raise ValueError('--prefix or --output is needed.')
    for i in listdir(d):
        if not i.endswith('.po'):
            continue
        fn = join(d, i)
        out = None
        if arg.output:
            out = join(arg.output, i.rstrip(".po") + ".mo")
        elif arg.prefix:
            out = join(arg.prefix, 'share/locale', RE.search(i).group(1), 'LC_MESSAGES/pixiv_downloader.mo')
        if out is None:
            raise ValueError('Failed to get output file name.')
        out_dir = dirname(out)
        if not exists(out_dir):
            makedirs(out_dir, 644, exist_ok=True)
        if call('msgfmt', fn, '--output', out, '--no-hash') != 0:
            raise ValueError('Failed to run msgfmt.')
except Exception:
    print_exc()
    _exit(1)
