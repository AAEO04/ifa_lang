# -*- coding: utf-8 -*-
"""
╔══════════════════════════════════════════════════════════════════════════════╗
║                        ỌJÀ - THE IFÁ-LANG MARKET                             ║
║                      "Where Spirits Buy and Sell"                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║  A decentralized, Git-based package manager for Ifá-Lang.                    ║
║  Dependencies are stored in libs/ and tracked in ifa.toml.                   ║
║                                                                              ║
║  Commands:                                                                   ║
║    oja init <name>        - Initialize a new project                         ║
║    oja add <git_url>      - Download and register a dependency              ║
║    oja install / oja ra   - Sync all dependencies from ifa.toml             ║
║    oja remove <name> / ta - Remove a package                                 ║
║    oja list               - Show all installed packages                      ║
╚══════════════════════════════════════════════════════════════════════════════╝
"""

import os
import subprocess
import shutil
from typing import Dict, Optional

# Try to import toml, fallback to simple parser if not available
try:
    import toml
    TOML_AVAILABLE = True
except ImportError:
    TOML_AVAILABLE = False


# =============================================================================
# SIMPLE TOML PARSER (Zero-Dependency Fallback)
# =============================================================================

def simple_toml_parse(content: str) -> dict:
    """Minimal TOML parser for basic ifa.toml files."""
    result = {}
    current_section = result
    
    for line in content.split('\n'):
        line = line.strip()
        
        # Skip comments and empty lines
        if not line or line.startswith('#'):
            continue
        
        # Section header [section.name]
        if line.startswith('[') and line.endswith(']'):
            section_name = line[1:-1]
            parts = section_name.split('.')
            current_section = result
            for part in parts:
                if part not in current_section:
                    current_section[part] = {}
                current_section = current_section[part]
            continue
        
        # Key = Value
        if '=' in line:
            key, value = line.split('=', 1)
            key = key.strip()
            value = value.strip()
            
            # Remove quotes from strings
            if (value.startswith('"') and value.endswith('"')) or \
               (value.startswith("'") and value.endswith("'")):
                value = value[1:-1]
            
            current_section[key] = value
    
    return result


def simple_toml_dump(data: dict, indent: int = 0) -> str:
    """Minimal TOML writer."""
    lines = []
    prefix = ""
    
    # First pass: write non-dict values
    for key, value in data.items():
        if not isinstance(value, dict):
            if isinstance(value, str):
                lines.append(f'{key} = "{value}"')
            else:
                lines.append(f'{key} = {value}')
    
    # Second pass: write sections
    for key, value in data.items():
        if isinstance(value, dict):
            lines.append(f'\n[{key}]')
            for k, v in value.items():
                if isinstance(v, dict):
                    # Nested section
                    lines.append(f'\n[{key}.{k}]')
                    for kk, vv in v.items():
                        if isinstance(vv, str):
                            lines.append(f'{kk} = "{vv}"')
                        else:
                            lines.append(f'{kk} = {vv}')
                else:
                    if isinstance(v, str):
                        lines.append(f'{k} = "{v}"')
                    else:
                        lines.append(f'{k} = {v}')
    
    return '\n'.join(lines)


# =============================================================================
# ỌJÀ MARKET CLASS
# =============================================================================

import hashlib
from datetime import datetime


class OjaMarket:
    """
    The Ọjà (Market) - Package Manager for Ifá-Lang.
    
    Features:
    - Git-based decentralized packages (no central registry needed)
    - Tracks dependencies in ifa.toml
    - Downloads to libs/ directory
    - Package verification via ifa.lock (SHA256 + commit hash)
    - Supports semantic versioning (future)
    """
    
    def __init__(self, project_dir: str = "."):
        self.project_dir = project_dir
        self.manifest_file = os.path.join(project_dir, "ifa.toml")
        self.lock_file = os.path.join(project_dir, "ifa.lock")
        self.lib_dir = os.path.join(project_dir, "libs")
    
    # =========================================================================
    # MANIFEST (ifa.toml) OPERATIONS
    # =========================================================================
    
    def _load_manifest(self) -> dict:
        """Load and parse ifa.toml."""
        if not os.path.exists(self.manifest_file):
            return {}
        
        with open(self.manifest_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if TOML_AVAILABLE:
            return toml.loads(content)
        else:
            return simple_toml_parse(content)
    
    def _save_manifest(self, data: dict):
        """Save data to ifa.toml."""
        with open(self.manifest_file, 'w', encoding='utf-8') as f:
            if TOML_AVAILABLE:
                toml.dump(data, f)
            else:
                f.write(simple_toml_dump(data))
    
    def _ensure_manifest(self):
        """Create ifa.toml if it doesn't exist."""
        if not os.path.exists(self.manifest_file):
            print("📜 Creating new ifa.toml...")
            self.init("my-project")
    
    def _ensure_lib_dir(self):
        """Create libs/ directory if it doesn't exist."""
        if not os.path.exists(self.lib_dir):
            os.makedirs(self.lib_dir)
            print(f"📁 Created {self.lib_dir}/")
    
    # =========================================================================
    # LOCK FILE (ifa.lock) - Package Verification
    # =========================================================================
    
    def _load_lock(self) -> dict:
        """Load ifa.lock file."""
        if not os.path.exists(self.lock_file):
            return {"packages": {}}
        
        with open(self.lock_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        if TOML_AVAILABLE:
            return toml.loads(content)
        else:
            return simple_toml_parse(content)
    
    def _save_lock(self, data: dict):
        """Save ifa.lock file."""
        with open(self.lock_file, 'w', encoding='utf-8') as f:
            if TOML_AVAILABLE:
                toml.dump(data, f)
            else:
                f.write(simple_toml_dump(data))
    
    def _get_git_commit(self, pkg_path: str) -> Optional[str]:
        """Get current Git commit SHA for a package."""
        try:
            result = subprocess.run(
                ["git", "-C", pkg_path, "rev-parse", "HEAD"],
                capture_output=True, text=True
            )
            if result.returncode == 0:
                return result.stdout.strip()
        except:
            pass
        return None
    
    def _compute_checksum(self, pkg_path: str) -> str:
        """Compute SHA256 checksum of package contents."""
        sha256 = hashlib.sha256()
        
        for root, dirs, files in os.walk(pkg_path):
            # Skip .git directory
            dirs[:] = [d for d in dirs if d != '.git']
            
            for filename in sorted(files):
                filepath = os.path.join(root, filename)
                try:
                    with open(filepath, 'rb') as f:
                        for chunk in iter(lambda: f.read(4096), b''):
                            sha256.update(chunk)
                except:
                    pass
        
        return sha256.hexdigest()
    
    def _lock_package(self, name: str, url: str):
        """Add/update a package in ifa.lock with verification data."""
        target_path = os.path.join(self.lib_dir, name)
        
        if not os.path.exists(target_path):
            return
        
        lock = self._load_lock()
        if "packages" not in lock:
            lock["packages"] = {}
        
        commit = self._get_git_commit(target_path)
        checksum = self._compute_checksum(target_path)
        
        lock["packages"][name] = {
            "url": url,
            "commit": commit or "unknown",
            "sha256": checksum,
            "locked_at": datetime.now().isoformat()
        }
        
        self._save_lock(lock)
        print(f"   🔒 Locked {name} (sha256: {checksum[:16]}...)")
    
    def verify(self, name: Optional[str] = None) -> bool:
        """
        Verify package integrity against ifa.lock.
        Returns True if all packages pass verification.
        """
        if not os.path.exists(self.lock_file):
            print("⚠️  No ifa.lock found. Run 'ifa oja lock' to generate.")
            return False
        
        lock = self._load_lock()
        packages = lock.get("packages", {})
        
        if not packages:
            print("📭 No packages in ifa.lock to verify.")
            return True
        
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║               🔐 ỌJÀ SECURITY - Verifying Packages           ║
╚══════════════════════════════════════════════════════════════╝
""")
        
        if name:
            packages = {name: packages.get(name)} if name in packages else {}
        
        passed = 0
        failed = 0
        missing = 0
        
        for pkg_name, pkg_info in packages.items():
            if not pkg_info:
                continue
                
            target_path = os.path.join(self.lib_dir, pkg_name)
            
            if not os.path.exists(target_path):
                print(f"   ❌ {pkg_name}: NOT INSTALLED")
                missing += 1
                continue
            
            # Verify checksum
            expected_checksum = pkg_info.get("sha256", "")
            actual_checksum = self._compute_checksum(target_path)
            
            # Verify commit
            expected_commit = pkg_info.get("commit", "")
            actual_commit = self._get_git_commit(target_path) or ""
            
            checksum_ok = expected_checksum == actual_checksum
            commit_ok = expected_commit == actual_commit or expected_commit == "unknown"
            
            if checksum_ok and commit_ok:
                print(f"   ✅ {pkg_name}: VERIFIED")
                passed += 1
            else:
                print(f"   ⚠️  {pkg_name}: MISMATCH")
                if not checksum_ok:
                    print(f"      └── Checksum: expected {expected_checksum[:16]}..., got {actual_checksum[:16]}...")
                if not commit_ok:
                    print(f"      └── Commit: expected {expected_commit[:8]}..., got {actual_commit[:8]}...")
                failed += 1
        
        print(f"""
   ════════════════════════════════════════════════════════════
   📊 Results: {passed} passed, {failed} failed, {missing} missing
""")
        
        if failed > 0:
            print("   ⚠️  WARNING: Some packages have been modified!")
            print("   Run 'ifa oja install --clean' to restore from source.")
        
        return failed == 0
    
    def lock(self):
        """Generate/update ifa.lock for all installed packages."""
        if not os.path.exists(self.manifest_file):
            print("❌ No ifa.toml found.")
            return False
        
        manifest = self._load_manifest()
        deps = manifest.get("dependencies", {})
        
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║               🔒 ỌJÀ - Locking Package Versions              ║
╚══════════════════════════════════════════════════════════════╝
""")
        
        for name, url in deps.items():
            target_path = os.path.join(self.lib_dir, name)
            if os.path.exists(target_path):
                self._lock_package(name, url)
            else:
                print(f"   ⚠️  {name}: not installed, skipping")
        
        print(f"\n   📝 Lock file saved to ifa.lock")
        return True
    
    # =========================================================================
    # MARKET COMMANDS
    # =========================================================================
    
    def init(self, name: str = "my-project", description: str = ""):
        """
        Initialize a new Ifá-Lang project.
        Creates ifa.toml with default configuration.
        """
        if os.path.exists(self.manifest_file):
            print("⚠️  ifa.toml already exists.")
            return False
        
        manifest = {
            "package": {
                "name": name,
                "version": "0.1.0",
                "description": description or "A new Ifá-Lang project",
                "authors": [],
            },
            "dependencies": {},
        }
        
        self._save_manifest(manifest)
        self._ensure_lib_dir()
        
        # Create basic project structure
        src_dir = os.path.join(self.project_dir, "src")
        if not os.path.exists(src_dir):
            os.makedirs(src_dir)
        
        # Create main.ifa
        main_file = os.path.join(src_dir, "main.ifa")
        if not os.path.exists(main_file):
            with open(main_file, 'w', encoding='utf-8') as f:
                f.write(f'''// {name} - Main Entry Point
iba std.irosu;

Irosu.fo("Ẹ káàbọ̀! Welcome to {name}!");

ase;
''')
        
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║               ✅ PROJECT INITIALIZED                         ║
╚══════════════════════════════════════════════════════════════╝

   📦 Project: {name}
   📁 Structure:
      ├── ifa.toml        (manifest)
      ├── libs/           (dependencies)
      └── src/
          └── main.ifa    (entry point)

   🚀 Next steps:
      ifa run src/main.ifa    # Run your program
      ifa oja add <git_url>   # Add a dependency
""")
        return True
    
    def add(self, url: str, name: Optional[str] = None):
        """
        Add a dependency from a Git URL.
        Downloads to libs/ and updates ifa.toml.
        """
        self._ensure_manifest()
        self._ensure_lib_dir()
        
        # Deduce name from URL if not provided
        if not name:
            name = url.split("/")[-1].replace(".git", "")
        
        target_path = os.path.join(self.lib_dir, name)
        
        # Check if already installed
        if os.path.exists(target_path):
            print(f"⚠️  Package '{name}' already exists locally. Use 'oja update' to refresh.")
        else:
            print(f"\n📦 Downloading '{name}' from Ọjà Market...")
            print(f"   Source: {url}")
            
            try:
                # Clone with depth=1 for speed
                subprocess.check_call(
                    ["git", "clone", url, target_path, "--depth", "1"],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL
                )
                print(f"   ✅ Installed to libs/{name}/")
            except subprocess.CalledProcessError:
                print(f"   ❌ Failed to download from {url}")
                print(f"      Check that the URL is correct and git is installed.")
                return False
            except FileNotFoundError:
                print(f"   ❌ Git command not found. Please install git.")
                return False
        
        # Update ifa.toml
        manifest = self._load_manifest()
        if "dependencies" not in manifest:
            manifest["dependencies"] = {}
        
        if name not in manifest["dependencies"]:
            manifest["dependencies"][name] = url
            self._save_manifest(manifest)
            print(f"   📝 Added '{name}' to ifa.toml")
        
        return True
    
    def install(self):
        """
        Install all dependencies listed in ifa.toml.
        Also known as 'ra' (to buy in Yoruba).
        """
        if not os.path.exists(self.manifest_file):
            print("❌ No ifa.toml found. Run 'ifa oja init <name>' first.")
            return False
        
        manifest = self._load_manifest()
        deps = manifest.get("dependencies", {})
        
        if not deps:
            print("📭 No dependencies found in ifa.toml.")
            return True
        
        self._ensure_lib_dir()
        
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║               ỌJÀ MARKET - Syncing Packages                  ║
╚══════════════════════════════════════════════════════════════╝
""")
        
        success_count = 0
        fail_count = 0
        
        for name, url in deps.items():
            target_path = os.path.join(self.lib_dir, name)
            
            if os.path.exists(target_path):
                print(f"   ✅ {name} (already installed)")
                success_count += 1
            else:
                print(f"   ⬇️  Downloading {name}...")
                try:
                    subprocess.check_call(
                        ["git", "clone", url, target_path, "--depth", "1"],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL
                    )
                    print(f"   ✅ {name}")
                    success_count += 1
                except (subprocess.CalledProcessError, FileNotFoundError):
                    print(f"   ❌ {name} (failed)")
                    fail_count += 1
        
        print(f"\n   📊 Result: {success_count} installed, {fail_count} failed")
        return fail_count == 0
    
    def remove(self, name: str):
        """
        Remove a package.
        Also known as 'ta' (to sell/discard in Yoruba).
        """
        # Remove from disk
        target_path = os.path.join(self.lib_dir, name)
        if os.path.exists(target_path):
            shutil.rmtree(target_path)
            print(f"🗑️  Removed libs/{name}/")
        else:
            print(f"⚠️  Package '{name}' not found in libs/")
        
        # Remove from ifa.toml
        if os.path.exists(self.manifest_file):
            manifest = self._load_manifest()
            if "dependencies" in manifest and name in manifest["dependencies"]:
                del manifest["dependencies"][name]
                self._save_manifest(manifest)
                print(f"📝 Removed '{name}' from ifa.toml")
        
        return True
    
    def update(self, name: Optional[str] = None):
        """
        Update a package (or all packages) to latest version.
        """
        if not os.path.exists(self.manifest_file):
            print("❌ No ifa.toml found.")
            return False
        
        manifest = self._load_manifest()
        deps = manifest.get("dependencies", {})
        
        if name:
            # Update single package
            if name not in deps:
                print(f"⚠️  Package '{name}' not in dependencies.")
                return False
            packages = {name: deps[name]}
        else:
            packages = deps
        
        print(f"🔄 Updating {len(packages)} package(s)...")
        
        for pkg_name, url in packages.items():
            target_path = os.path.join(self.lib_dir, pkg_name)
            if os.path.exists(target_path):
                print(f"   🔄 Updating {pkg_name}...")
                try:
                    subprocess.check_call(
                        ["git", "-C", target_path, "pull", "--ff-only"],
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL
                    )
                    print(f"   ✅ {pkg_name} updated")
                except subprocess.CalledProcessError:
                    print(f"   ⚠️  {pkg_name} has local changes, skipped")
            else:
                print(f"   ⬇️  {pkg_name} not installed, cloning...")
                self.add(url, pkg_name)
        
        return True
    
    def list(self):
        """
        List all installed packages.
        """
        manifest = self._load_manifest()
        deps = manifest.get("dependencies", {})
        pkg_info = manifest.get("package", {})
        
        print(f"""
╔══════════════════════════════════════════════════════════════╗
║                    ỌJÀ MARKET - Inventory                    ║
╚══════════════════════════════════════════════════════════════╝

   📦 Project: {pkg_info.get('name', 'Unknown')} v{pkg_info.get('version', '0.0.0')}
""")
        
        if not deps:
            print("   📭 No dependencies installed.")
        else:
            print(f"   📚 Dependencies ({len(deps)}):")
            for name, url in deps.items():
                target_path = os.path.join(self.lib_dir, name)
                status = "✅" if os.path.exists(target_path) else "❌"
                print(f"      {status} {name}")
                print(f"         └── {url}")
        
        print()
        return True


# =============================================================================
# CLI INTERFACE
# =============================================================================

def oja_cli(args: list):
    """Command-line interface for Ọjà."""
    market = OjaMarket()
    
    if not args:
        print("""
╔══════════════════════════════════════════════════════════════╗
║                    ỌJÀ - THE MARKET                          ║
║              Package Manager for Ifá-Lang                    ║
╚══════════════════════════════════════════════════════════════╝

Usage:
    ifa oja init <name>         Create a new project
    ifa oja add <git_url>       Add a dependency
    ifa oja install (or 'ra')   Install all deps from ifa.toml
    ifa oja remove <name> (ta)  Remove a package
    ifa oja update [name]       Update package(s)
    ifa oja list                Show installed packages
    ifa oja lock                Generate ifa.lock with checksums
    ifa oja verify [name]       Verify package integrity
""")
        return
    
    command = args[0].lower()
    
    if command == "init":
        name = args[1] if len(args) > 1 else "my-project"
        market.init(name)
    
    elif command == "add":
        if len(args) < 2:
            print("Usage: ifa oja add <git_url> [name]")
            return
        url = args[1]
        name = args[2] if len(args) > 2 else None
        market.add(url, name)
    
    elif command in ("install", "ra"):  # ra = buy
        market.install()
    
    elif command in ("remove", "ta"):  # ta = sell/discard
        if len(args) < 2:
            print("Usage: ifa oja remove <name>")
            return
        market.remove(args[1])
    
    elif command == "update":
        name = args[1] if len(args) > 1 else None
        market.update(name)
    
    elif command == "list":
        market.list()
    
    elif command == "lock":
        market.lock()
    
    elif command == "verify":
        name = args[1] if len(args) > 1 else None
        market.verify(name)
    
    else:
        print(f"❌ Unknown command: {command}")
        print("   Run 'ifa oja' for help.")


# =============================================================================
# MAIN
# =============================================================================

if __name__ == "__main__":
    import sys
    oja_cli(sys.argv[1:])
