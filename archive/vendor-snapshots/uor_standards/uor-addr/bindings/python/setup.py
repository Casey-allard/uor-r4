"""Setuptools entry point used only to coerce `bdist_wheel` into
producing a *platform-tagged + ctypes-friendly* wheel.

The package's metadata lives in `pyproject.toml`; this file exists
purely to fix the wheel-tagging defaults:

- **Without intervention**, setuptools tags the wheel `py3-none-any`
  (pure-Python). That's wrong: we bundle a per-platform native
  library (`libuor_addr_c.{so,dylib,dll}`) and the wheel must carry
  a platform-specific tag so pip refuses to install it on the wrong
  OS / arch.
- **`has_ext_modules = True`** is the canonical setuptools knob for
  "this distribution carries a compiled artifact." It forces the
  package files into `platlib` (where shared libraries belong) and
  flips the wheel filename to encode the current Python's ABI. That's
  *too* restrictive for us — we only use `ctypes` from stdlib, so any
  CPython 3.x can load the same wheel.
- **`bdist_wheel.get_tag()`** is overridden to keep the python-tag
  at `py3` and the ABI at `none` while preserving the platform tag
  the platlib-aware build chose. Net wheel tag: `py3-none-<plat>` —
  platform-specific, Python-version-agnostic.

Linux wheels also go through `auditwheel repair` in the release
workflow to rebrand `linux_*` → `manylinux_*_*` per PEP 600. The
.so being in platlib (not purelib) is what makes `auditwheel
repair` accept the wheel.
"""

from setuptools import setup
from setuptools.dist import Distribution


class _PlatformDistribution(Distribution):
    """Forces platlib placement so the bundled native library lands
    in the right directory of the installed wheel + `auditwheel
    repair` accepts the wheel for manylinux rebranding.

    Setuptools' default packaging routes everything into `purelib`
    unless the distribution claims to have an extension module —
    even when `bdist_wheel.root_is_pure = False`. Overriding this
    one method is the documented way to opt into platlib without
    actually defining a compiled extension.
    """

    def has_ext_modules(self):  # type: ignore[override]
        return True


try:
    try:
        from setuptools.command.bdist_wheel import bdist_wheel as _BdistWheelBase
    except ImportError:
        from wheel.bdist_wheel import bdist_wheel as _BdistWheelBase

    class _BdistWheel(_BdistWheelBase):
        """Force `py3-none-<plat>` instead of `cpX-cpX-<plat>` — we
        use `ctypes` from stdlib; the same wheel works for every
        CPython 3.x ABI on the same platform.
        """

        def get_tag(self):  # type: ignore[override]
            _python, _abi, plat = super().get_tag()
            return ("py3", "none", plat)

    cmdclass = {"bdist_wheel": _BdistWheel}
except ImportError:
    cmdclass = {}


setup(distclass=_PlatformDistribution, cmdclass=cmdclass)
