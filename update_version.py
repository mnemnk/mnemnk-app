from pathlib import Path
import platform
import re
import subprocess
import sys


def update_version(file_path, new_version):
    path = Path(file_path)
    content = path.read_text()
    if 'package.json' in file_path:
        content = re.sub(r'"version": "[^"]+"', f'"version": "{new_version}"', content)
    elif 'src-tauri/Cargo.toml' in file_path:
        content = re.sub(r'^version = "[^"]+"', f'version = "{new_version}"', content, flags=re.MULTILINE)
    elif 'src-tauri/tauri.conf.json' in file_path:
        content = re.sub(r'"version": "[^"]+"', f'"version": "{new_version}"', content)
    path.write_text(content, newline='\n')

    subprocess.run(["git", "add", file_path], check=True)
    print(f"{file_path} has been updated to {new_version}.")


def test_build():
    if platform.system() == "Windows":
        subprocess.run(["powershell.exe", "-Command", "npm run tauri build"], check=True)
    else:
        subprocess.run(["npm", "run", "tauri", "build"], check=True)
    subprocess.run(["git", "add", "package-lock.json"], check=True)
    subprocess.run(["git", "add", "src-tauri/Cargo.lock"], check=True)
    print("Build has been tested.")


def set_git_tag(new_version):
    tag_name = f"v{new_version}"
    # It seems Cargo.lock change is not added in test_build() so we add it again here just in case.
    subprocess.run(["git", "add", "src-tauri/Cargo.lock"], check=True)
    subprocess.run(["git", "commit", "-m", f"chore: release {tag_name}"], check=True)
    subprocess.run(["git", "tag", tag_name], check=True)
    print(f"Git tag {tag_name} has been created.")


def main(new_version):
    update_version("package.json", new_version)
    update_version("src-tauri/Cargo.toml", new_version)
    update_version("src-tauri/tauri.conf.json", new_version)
    test_build()
    set_git_tag(new_version)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python update_version.py <new_version>")
        sys.exit(1)
    main(sys.argv[1])
