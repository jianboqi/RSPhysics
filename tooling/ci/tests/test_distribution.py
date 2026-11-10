"""Release gates must reject inconsistent or incomplete artifacts."""

from email.message import EmailMessage
import importlib.util
import io
from pathlib import Path
import tarfile
import tomllib
from tempfile import TemporaryDirectory
import unittest
from unittest.mock import patch
import zipfile

spec = importlib.util.spec_from_file_location("verify_distribution", Path(__file__).resolve().parents[1] / "verify_distribution.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)
VERSION = tomllib.loads((Path(__file__).resolve().parents[3] / 'bindings/python/pyproject.toml').read_text(encoding='utf8'))['project']['version']


def metadata(version=VERSION, license_files=True, numpy=True):
    message = EmailMessage()
    message["Metadata-Version"] = "2.4"
    message["Name"] = "rsphysics"
    message["Version"] = version
    message["License-Expression"] = "Apache-2.0 AND MIT"
    if license_files:
        message["License-File"] = "LICENSE"
    if numpy:
        message["Requires-Dist"] = "numpy>=1.26"
    return message.as_bytes()


class DistributionTests(unittest.TestCase):
    def test_metadata(self):
        release.check_metadata(metadata(), "wheel", VERSION)

    def test_wrong_version(self):
        with self.assertRaisesRegex(ValueError, "name/version"):
            release.check_metadata(metadata("9.9.9"), "wheel", VERSION)

    def test_missing_license_files(self):
        with self.assertRaisesRegex(ValueError, "license"):
            release.check_metadata(metadata(license_files=False), "wheel", VERSION)

    def test_missing_dependency(self):
        with self.assertRaisesRegex(ValueError, "NumPy"):
            release.check_metadata(metadata(numpy=False), "wheel", VERSION)

    def test_wrong_tag(self):
        with patch("sys.argv", ["verify_distribution", "--tag", "v9.9.9"]):
            with self.assertRaisesRegex(ValueError, "Release tag"):
                release.main()

    def bundle(self, path):
        with zipfile.ZipFile(path / "rsphysics-0.1.0-cp313-cp313-win_amd64.whl", "w") as archive:
            archive.writestr("rsphysics-0.1.0.dist-info/METADATA", metadata())
        with tarfile.open(path / "rsphysics-0.1.0.tar.gz", "w:gz") as archive:
            data = metadata()
            entry = tarfile.TarInfo("rsphysics-0.1.0/PKG-INFO")
            entry.size = len(data)
            archive.addfile(entry, io.BytesIO(data))

    def test_complete_bundle(self):
        with TemporaryDirectory() as directory:
            self.bundle(Path(directory))
            with patch("sys.argv", ["verify_distribution", "--dist", directory, "--expected-wheels", "1", "--tag", f"v{VERSION}"]):
                release.main()

    def test_missing_wheel(self):
        with TemporaryDirectory() as directory:
            self.bundle(Path(directory))
            with patch("sys.argv", ["verify_distribution", "--dist", directory, "--expected-wheels", "25", "--tag", ""]):
                with self.assertRaisesRegex(ValueError, "Expected 25 wheels"):
                    release.main()

    def test_missing_source(self):
        with TemporaryDirectory() as directory:
            with patch("sys.argv", ["verify_distribution", "--dist", directory, "--tag", ""]):
                with self.assertRaisesRegex(ValueError, "source archive"):
                    release.main()


if __name__ == "__main__":
    unittest.main()
