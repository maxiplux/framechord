#!/usr/bin/env python3
"""Build and verify a native local distribution. No publication or provider calls."""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--offline', action='store_true', help='Use cached Rust dependencies only')
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
manifest = json.loads(subprocess.run(['cargo', 'metadata', '--format-version', '1', '--no-deps', '--locked', '--offline', '--manifest-path', str(root / 'Cargo.toml')], capture_output=True, text=True, check=True).stdout)
package = next(package for package in manifest['packages'] if package['name'] == 'framechord')
version = package['version']
rust = subprocess.run(['rustc', '-vV'], capture_output=True, text=True, check=True).stdout
host = next(line.split(': ', 1)[1] for line in rust.splitlines() if line.startswith('host: '))
command = ['cargo', 'build', '--release', '--locked', '--manifest-path', str(root / 'Cargo.toml'), '--target-dir', str(root / 'target')]
if args.offline:
    command.append('--offline')
subprocess.run(command, cwd=root, check=True)
executable = 'framechord.exe' if os.name == 'nt' else 'framechord'
binary = root / 'target/release' / executable
credentials = ['OPENAI_API_KEY', 'GEMINI_API_KEY', 'FISH_API_KEY', 'OPENROUTER_API_KEY', 'ELEVENLABS_API_KEY']
# These subprocesses exercise only offline commands and never receive live keys.
smoke_environment = {key: value for key, value in os.environ.items() if key not in credentials}
checks = []
with tempfile.TemporaryDirectory(prefix='framechord-package-') as temporary:
    isolated = pathlib.Path(temporary)
    copied = isolated / executable
    shutil.copy2(binary, copied)
    project = isolated / 'project with spaces'
    project.mkdir()
    preserved = 'LOCAL_FIXTURE=preserve-existing-file\n'
    (project / '.env').write_text(preserved)
    def cli(arguments):
        result = subprocess.run([str(copied), '--json', '--non-interactive'] + arguments, cwd=isolated, env=smoke_environment, capture_output=True, text=True, check=True)
        response = json.loads(result.stdout)
        if not response['ok']:
            raise RuntimeError('Offline binary verification failed')
        return response
    # A downloaded Unix executable is itself named framechord. Its directory
    # cannot also contain the required framechord/ project configuration folder.
    blocker = isolated / 'framechord'
    if not blocker.exists():
        blocker.write_text('fixture name collision')
    blocker_bytes = blocker.read_bytes()
    collision = subprocess.run([str(copied), 'init', '--json', '--non-interactive'], cwd=isolated, env=smoke_environment, capture_output=True, text=True)
    assert collision.returncode == 2
    assert json.loads(collision.stdout)['errors'][0]['code'] == 'init_path_conflict'
    assert blocker.read_bytes() == blocker_bytes
    assert not (isolated / '.env').exists()
    checks.append('initialization_rejects_executable_config_path_collision')
    cli(['init', str(project), '--preset', 'empty'])
    cli(['init', str(project), '--preset', 'empty'])
    assert (project / '.env').read_text() == preserved + 'GEMINI_API_KEY=\nOPENAI_API_KEY=\nFISH_API_KEY=\nOPENROUTER_API_KEY=\nELEVENLABS_API_KEY=\n'
    checks.append('offline_repeat_initialization_preserves_env')
    for agent in ['.agents', '.claude']:
        skill = project / agent / 'skills/framechord'
        assert (skill / 'SKILL.md').is_file()
        assert (skill / 'references/voices.md').is_file()
        assert (skill / 'references/schemas/operation.json').is_file()
        assert (skill / 'references/schemas/voice_design.json').is_file()
    checks.append('embedded_codex_and_claude_skills_without_checkout')
    cli(['--project', str(project), 'skills', 'install', '--agents', 'copilot,gemini'])
    assert (project / '.github/skills/framechord/SKILL.md').is_file()
    assert (project / '.gemini/skills/framechord/SKILL.md').is_file()
    assert not (project / '.cursor').exists()
    checks.append('native_agent_targets_without_checkout')
    recommended_project = isolated / 'recommended-project'
    recommended = cli(['init', str(recommended_project), '--preset', 'recommended'])['data']
    assert recommended['configuration']['defaults']['image']['model'] == 'gemini-nano-banana-2.1'
    assert recommended['configuration']['defaults']['speech']['provider'] == 'google'
    assert recommended['configuration']['default_settings']['speech']['voice'] == 'Kore'
    assert recommended['configuration']['defaults']['music']['model'] == 'lyria-3.5'
    assert recommended['setup_complete'] is False and recommended['pending_steps']
    readiness = cli(['--project', str(recommended_project), 'config', 'status'])['data']
    assert readiness['setup_complete'] is False and readiness['account_access_verified'] is False
    assert (project / '.agents/skills/framechord/references/setup.md').is_file()
    checks.append('recommended_init_and_resumable_setup_readiness')
    # A renamed installed executable must keep pre-rename project paths intact.
    legacy = isolated / 'legacy-project'
    cli(['init', str(legacy), '--preset', 'empty'])
    (legacy / 'framechord').rename(legacy / 'mediaagent')
    legacy_config = (legacy / 'mediaagent/config.yaml').read_bytes()
    legacy_persona = (legacy / 'mediaagent/characters/persona/character.yaml').read_bytes()
    result = cli(['init', str(legacy)])['data']
    assert result['legacy_layout'] is True and pathlib.Path(result['data_directory']).samefile(legacy / 'mediaagent')
    assert not (legacy / 'framechord').exists()
    assert (legacy / 'mediaagent/config.yaml').read_bytes() == legacy_config
    assert (legacy / 'mediaagent/characters/persona/character.yaml').read_bytes() == legacy_persona
    assert (legacy / '.agents/skills/framechord/SKILL.md').is_file()
    checks.append('legacy_project_paths_identity_and_configuration_preserved')
    exported = isolated / 'portable-skill'
    cli(['--project', str(project), 'skills', 'export', str(exported)])
    assert (exported / 'templates/voice-preview.md').is_file()
    checks.append('portable_skill_export_without_checkout')
    out = cli(['--project', str(project), 'generate', 'image', '--provider', 'openai', '--model', 'gpt-image-2.5-flare', '--prompt', 'An offline dry-run leaf', '--output', str(isolated / 'leaf.png'), '--dry-run'])
    assert out['data']['dry_run'] and not (isolated / 'leaf.png').exists()
    assert not list((project / 'framechord/runs').glob('*/run.yaml'))
    checks.append('offline_direct_generation_dry_run')
    schema = cli(['schema', 'voice_design'])['data']
    assert schema['additionalProperties'] is False
    checks.append('embedded_strict_schemas')
    identity = cli(['--project', str(project), 'persona', 'show', 'persona'])['data']
    assert identity['visual'] is None and identity['voice'] is None
    checks.append('initial_persona_keeps_profiles_disabled')
    configured = isolated / 'configured-project'
    initial = cli(['init', str(configured), '--defaults-file', str(exported / 'templates/project-config.yaml')])
    assert initial['data']['configuration']['consistency']['require_known_identity_model'] is True
    identity = cli(['--project', str(configured), 'persona', 'show', 'persona'])['data']
    assert identity['visual']['selection'] == initial['data']['configuration']['defaults']['image']
    assert identity['visual']['settings'] == initial['data']['configuration']['default_settings']['image']
    assert identity['voice']['binding'] == initial['data']['configuration']['default_settings']['speech']['voice']
    resolved = cli(['--project', str(configured), 'generate', 'image', '--prompt', 'An offline original', '--dry-run'])
    assert resolved['data']['inputs'][0]['selection'] == identity['visual']['selection']
    assert not resolved['data']['inputs'][0]['settings'].get('input_fidelity')
    checks.append('complete_init_settings_seed_profiles_and_generation_defaults')
    diagnostic = cli(['--project', str(project), 'doctor'])['data']
    assert diagnostic['live_requested'] is False and diagnostic['network_checked'] is False
    assert len(diagnostic['credentials']) == 5 and diagnostic['generation_performed'] is False
    assert (exported / 'references/connections.md').is_file()
    assert (exported / 'references/schemas/diagnostics.json').is_file()
    checks.append('offline_doctor_and_embedded_connection_contract')
    # A generic process caller exercises new contracts in a path containing spaces.
    jobs = project / 'acceptance jobs'
    jobs.mkdir()
    (jobs / 'brief.md').write_text('One clear leaf on a plain background.')
    (jobs / 'job.yaml').write_text('schema_version: 2\nid: package-check\nmedia: image\nselection: {provider: openai, model: gpt-image-2.5-flare}\nbrief: brief.md\n')
    (jobs / 'suite.yaml').write_text('schema_version: 1\nid: package-check\ndescription: Isolated offline preview\njobs: [job.yaml]\nlimits: {requests: 1, images: 1, audio_seconds: 60, speech_chars: 1000, revisions: 1, attempts: 1, concurrency: 1, per_provider: 1}\ncriteria: [Human content acceptance]\n')
    preview = cli(['--project', str(project), 'verify', '--suite', str(jobs / 'suite.yaml')])['data']
    assert preview['estimated_reservation']['requests'] == 1
    assert len(preview['preview_hash']) == 64 and preview['generation_performed'] is False
    assert not list((project / 'framechord/runs').glob('*/run.yaml'))
    checks.append('bounded_verification_preview_without_provider_calls')
    before = (jobs / 'job.yaml').read_bytes()
    cli(['--project', str(project), 'migrate', '--kind', 'job', '--file', str(jobs / 'job.yaml')])
    assert (jobs / 'job.yaml').read_bytes() == before
    assert not (jobs / 'job.v1.bak').exists()
    checks.append('migration_preview_preserves_authored_files')
    report = cli(['--project', str(project), 'usage'])['data']
    assert report['currency_total'] is None
    assert report['estimate_summary']['estimated_jobs'] == 0 and report['estimated_total_usd'] is None
    cleanup = isolated / 'cleanup.yaml'
    cli(['--project', str(project), 'storage', 'plan', '--output', str(cleanup)])
    assert cleanup.exists()
    checks.append('usage_and_conservative_storage_preview')
    for schema_name in ['job', 'run', 'verification']:
        assert (exported / 'references/schemas' / (schema_name + '.json')).exists()
    assert (exported / 'templates/cast-scene.yaml').exists()
    assert (exported / 'references/remediation.md').exists()
    checks.append('embedded_v2_cast_verification_and_recovery_guidance')
    installed_bin = isolated / 'installed bin'
    for _ in range(2):
        installation = cli(['install', '--bin-dir', str(installed_bin), '--no-modify-path'])['data']
        assert installation['project_files_changed'] is False and installation['network_requested'] is False
    installed = installed_bin / executable
    assert subprocess.run([str(installed), '--version'], capture_output=True, text=True, check=True).stdout.strip() == f'framechord {version}'
    assert (project / '.env').read_text() == preserved + 'GEMINI_API_KEY=\nOPENAI_API_KEY=\nFISH_API_KEY=\nOPENROUTER_API_KEY=\nELEVENLABS_API_KEY=\n'
    checks.append('offline_user_install_repeat_and_version_without_project_changes')
    subprocess.run([sys.executable, str(root / 'scripts/test-installers.py'), '--binary', str(copied), '--target', host], env=smoke_environment, check=True)
    checks.append('native_bootstrap_download_install_repeat_and_checksum_rejection_offline')
    reported = subprocess.run([str(copied), '--version'], cwd=isolated, env=smoke_environment, capture_output=True, text=True, check=True).stdout.strip()
    assert reported == f'framechord {version}'

name = f'framechord-{version}-{host}'
destination = root / 'target/package-dist'
destination.mkdir(parents=True, exist_ok=True)
source_commit = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
source_dirty = bool(subprocess.run(['git', 'status', '--porcelain'], cwd=root, capture_output=True, text=True, check=True).stdout.strip())
metadata = {'source_commit': source_commit, 'source_dirty': source_dirty, 'schema_version': 1, 'package': 'framechord', 'version': version, 'target': host,
            'binary_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
            'native_offline_checks': checks, 'publication_performed_by_packager': False,
            'live_provider_testing_performed_by_packager': False}
with tempfile.TemporaryDirectory(prefix='framechord-archive-') as temporary:
    staged = pathlib.Path(temporary) / name
    staged.mkdir()
    shutil.copy2(binary, staged / executable)
    (staged / '.env.example').write_text(''.join(f'{key}=\n' for key in credentials))
    (staged / 'verification.json').write_text(json.dumps(metadata, indent=2) + '\n')
    # LGPL notice for the statically linked LAME MP3 encoder; `framechord notices` prints the same text.
    shutil.copy2(root / 'assets/THIRD_PARTY_NOTICES.md', staged / 'THIRD_PARTY_NOTICES.md')
    (staged / 'README.md').write_text(f'''# framechord {version}

Native binary for `{host}`. From the download folder, run `./{executable} install` to install in your user bin directory without administrator access. Open a new terminal afterwards. Installation does not initialize a project or copy provider keys.

```sh
framechord --help
framechord update --check
framechord update
framechord update --rollback
framechord init /path/to/project
framechord models --json
framechord doctor --live --json  # Explicit read-only checks; no media generation
framechord skills export /path/to/portable-skill
```

Initialization is offline and preserves existing files. The binary embeds the maintained Codex/Claude skills, all command/flag help, YAML schemas, and authoring templates; no source checkout or Rust installation is needed at runtime. Use project-local skills by default; user installation is a separate explicit command.

Copy `.env.example` to your target project's `.env` and fill only the selected provider's credential, or use its process environment variable. Choose explicit provider/model/input/output settings before generation. Existing process values take precedence. No development credentials, generated media, or canonical characters are included in this package.

Images, infographics, speech, podcasts, audiobooks (per chapter or full book, WAV or MP3), music, sound effects, optional characters, concurrent jobs, budgets/cache/recovery, image reviews, and explicit ElevenLabs voice preview/create/bind workflows are available. Preview candidates need selection. Consent-gated ElevenLabs recorded-voice cloning, Fish audio-reference synthesis, Google audio review, and Google saved-interaction reconciliation are implemented with their documented boundaries; recording workflows have mocked coverage and require authorized samples for live verification. Offline initialization saves models/settings/voices/limits; original identity assets pin their recorded provider/model. Visual likeness remains subject to human acceptance. Video generation is excluded.

MP3 encoding uses the LAME library, statically linked under the LGPL; see `THIRD_PARTY_NOTICES.md` (also printed by `framechord notices`) for licenses, source and relinking.

`verification.json` records the binary hash and native offline checks. The packager does not publish; public availability is recorded on the GitHub Releases page. Binaries are not signed or notarized. Other platform packages require their own native build and verification. No provider generation is performed by the packager.
''')
    if os.name == 'nt':
        archive = destination / f'{name}.zip'
        with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as bundle:
            for file in sorted(staged.iterdir()):
                bundle.write(file, f'{name}/{file.name}')
        with zipfile.ZipFile(archive) as bundle:
            assert bundle.testzip() is None
            assert len(bundle.namelist()) == 5
    else:
        archive = destination / f'{name}.tar.gz'
        with tarfile.open(archive, 'w:gz') as bundle:
            bundle.add(staged, arcname=name)
        with tarfile.open(archive) as bundle:
            assert len(bundle.getmembers()) == 6
            member = bundle.extractfile(f'{name}/{executable}')
            assert hashlib.sha256(member.read()).hexdigest() == metadata['binary_sha256']
# Raw release assets avoid extraction tools in installers and the built-in updater.
raw = destination / (f'framechord-{host}' + ('.exe' if os.name == 'nt' else ''))
shutil.copy2(binary, raw)
raw.with_name(raw.name + '.sha256').write_text(f'{metadata["binary_sha256"]}  {raw.name}\n')
checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
checksum_file = archive.with_name(archive.name + '.sha256')
checksum_file.write_text(f'{checksum}  {archive.name}\n')
print(json.dumps({'archive': str(archive), 'checksum_file': str(checksum_file), 'sha256': checksum,
                  'native_checks_passed': len(checks), 'target': host, 'published': False}))
