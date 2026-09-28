import contextlib
import sys
import tempfile
import unittest
import urllib.error
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT = SCRIPT_DIR.parents[1]
sys.path.insert(0, str(SCRIPT_DIR))

import github_release
import registry_status
import resolve_release


def write_manifest(directory, version="1.2.3"):
    path = Path(directory) / "Cargo.toml"
    path.write_text(
        f'[package]\nname = "binlayout"\nversion = "{version}"\n'
    )
    return path


class GithubReleaseHelperTests(unittest.TestCase):
    def test_new_release_writes_tag_and_creates_release(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            output = Path(td) / "output"
            api_calls = []
            commands = []
            def fake_api(repository, api_path, allow_missing=False):
                api_calls.append((repository, api_path, allow_missing))
                return None

            def fake_runner(command, **kwargs):
                commands.append((command, kwargs))

            result = github_release.create_or_reuse_release(
                "owner/repo",
                "a" * 40,
                output,
                manifest_path=manifest,
                api_func=fake_api,
                runner=fake_runner,
            )

            self.assertEqual(result, "created")
            self.assertEqual(output.read_text(), "tag=v1.2.3\n")
            self.assertIn(("owner/repo", "releases/tags/v1.2.3", True), api_calls)
            self.assertEqual(commands[0][0][:4], ["gh", "release", "create", "v1.2.3"])
            self.assertIn("a" * 40, commands[0][0])

    def test_existing_release_still_writes_tag_and_skips_create(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            output = Path(td) / "output"
            def fake_api(repository, api_path, allow_missing=False):
                if api_path == "releases/tags/v1.2.3":
                    return {"tag_name": "v1.2.3"}
                self.fail(f"unexpected API path {api_path}")

            def fail_runner(*args, **kwargs):
                self.fail("existing release must not be recreated")

            result = github_release.create_or_reuse_release(
                "owner/repo",
                "b" * 40,
                output,
                manifest_path=manifest,
                api_func=fake_api,
                runner=fail_runner,
            )

            self.assertEqual(result, "existing")
            self.assertEqual(output.read_text(), "tag=v1.2.3\n")

    def test_existing_tag_must_match_tested_commit(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            output = Path(td) / "output"

            def fake_api(repository, api_path, allow_missing=False):
                if api_path.startswith("releases/tags/"):
                    return None
                if api_path.startswith("git/ref/tags/"):
                    return {"object": {"type": "commit", "sha": "c" * 40}}
                self.fail(f"unexpected API path {api_path}")
            with self.assertRaises(SystemExit):
                github_release.create_or_reuse_release(
                    "owner/repo",
                    "d" * 40,
                    output,
                    manifest_path=manifest,
                    api_func=fake_api,
                )


class ResolveReleaseTests(unittest.TestCase):
    def test_selected_tag_resolves_checked_out_commit(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            output = Path(td) / "output"
            commit = "e" * 40

            result = resolve_release.resolve_release(
                "v1.2.3",
                output,
                manifest_path=manifest,
                rev_parse=lambda: commit,
            )

            self.assertEqual(result, commit)
            self.assertEqual(output.read_text(), f"commit={commit}\n")

    def test_version_mismatch_fails_before_publish(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            output = Path(td) / "output"
            with self.assertRaises(SystemExit):
                resolve_release.resolve_release(
                    "v9.9.9",
                    output,
                    manifest_path=manifest,
                    rev_parse=lambda: self.fail("rev-parse must not run"),
                )


class RegistryStatusTests(unittest.TestCase):
    def test_existing_crate_is_published(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)

            def opener(request, timeout):
                return contextlib.nullcontext(object())

            self.assertTrue(registry_status.is_published(manifest, opener=opener))

    def test_404_means_not_yet_published(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)

            def opener(request, timeout):
                raise urllib.error.HTTPError(
                    request.full_url, 404, "Not Found", {}, None
                )

            self.assertFalse(registry_status.is_published(manifest, opener=opener))

    def test_registry_auth_and_server_errors_fail_visibly(self):
        with tempfile.TemporaryDirectory() as td:
            manifest = write_manifest(td)
            for status in (401, 500):
                with self.subTest(status=status):
                    def opener(request, timeout, status=status):
                        raise urllib.error.HTTPError(
                            request.full_url, status, "registry error", {}, None
                        )

                    with self.assertRaises(urllib.error.HTTPError):
                        registry_status.is_published(manifest, opener=opener)


class WorkflowStructureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.github_release = (ROOT / ".github/workflows/github-release.yml").read_text()
        cls.release = (ROOT / ".github/workflows/release.yml").read_text()
        cls.ci = (ROOT / ".github/workflows/ci.yml").read_text()

    def test_github_release_is_manual_only_and_links_publish_result(self):
        self.assertIn("on:\n  workflow_dispatch:", self.github_release)
        self.assertNotIn("\n  push:", self.github_release)
        self.assertNotIn("\n  pull_request:", self.github_release)
        self.assertIn("python3 .github/scripts/github_release.py", self.github_release)
        self.assertIn(
            "tag: ${{ steps.github-release.outputs.tag }}",
            self.github_release,
        )
        self.assertIn("publish:", self.github_release)
        self.assertIn("needs: release", self.github_release)
        self.assertIn("uses: ./.github/workflows/release.yml", self.github_release)
        self.assertIn(
            "tag: ${{ needs.release.outputs.tag }}",
            self.github_release,
        )
        self.assertIn("secrets: inherit", self.github_release)
        self.assertNotIn("gh workflow run release.yml", self.github_release)
        self.assertNotIn("actions: write", self.github_release)

    def test_publish_uses_selected_tag_source_and_requires_verification(self):
        self.assertIn("workflow_call:", self.release)
        self.assertIn(
            "ref: ${{ inputs.tag && format('refs/tags/{0}', inputs.tag) || github.ref }}",
            self.release,
        )
        self.assertIn("python3 .github/scripts/resolve_release.py", self.release)
        self.assertIn("needs: resolve", self.release)
        self.assertIn("ref: ${{ needs.resolve.outputs.commit }}", self.release)
        self.assertIn("needs: [resolve, verify]", self.release)
        self.assertIn("python3 .github/scripts/registry_status.py", self.release)
        self.assertIn(
            "if: steps.registry.outputs.published == 'false'",
            self.release,
        )

    def test_offline_release_tests_run_in_ci_without_credentials(self):
        self.assertIn(
            "python3 -m unittest discover -s .github/scripts -p 'test_*.py'",
            self.ci,
        )
        self.assertNotIn("CARGO_REGISTRY_TOKEN", self.ci)
        self.assertNotIn("GH_TOKEN", self.ci)


if __name__ == "__main__":
    unittest.main()
