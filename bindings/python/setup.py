from setuptools import Distribution, setup
from wheel.bdist_wheel import bdist_wheel


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


class PlatformWheel(bdist_wheel):
    def finalize_options(self):
        super().finalize_options()
        self.root_is_pure = False
        self.python_tag = "py3"

    def get_tag(self):
        python, _abi, plat = super().get_tag()
        return (python if python.startswith("py") else "py3", "none", plat)


setup(distclass=BinaryDistribution, cmdclass={"bdist_wheel": PlatformWheel})
