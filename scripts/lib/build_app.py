# encoding=utf8

import subprocess
import platform
import os
import shutil
import re

APP_NAME = "Etcd Workbench"
BUNDLE_NAME = "etcd-workbench"
ENCODING = "gbk" if platform.system().lower() == 'windows' else 'utf-8'

def execute(command, print_output = True):
    print(f"calling: {command}")
    subprocess.call(command, shell=True, encoding=ENCODING)
    
def build_app_windows(target, build_platform):
    os.chdir('../')
    root_path = os.getcwd()
    os.chdir('./app/')
    app_path = os.getcwd()

    bundle_path = os.path.join(app_path, f'src-tauri/target/{target}/release/bundle')
    if os.path.exists(bundle_path):
        shutil.rmtree(bundle_path)

    execute(f'pnpm tauri build --target {target}')

    to_dir = os.path.join(root_path, 'bin', 'app', build_platform)

    nsis_path = os.path.join(bundle_path, 'nsis')

    app_version = None
    for file in os.listdir(nsis_path):
        file_path = os.path.join(nsis_path, file)
        filename = None
        if file.endswith('.exe'):
            if app_version is None:
                app_version = re.findall(r".*(\d+\.\d+\.\d+).*", file)[0]
                print(f'parsed app version {app_version}')
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.exe"
        elif file.endswith('.nsis.zip'):
            if app_version is None:
                app_version = re.findall(r".*(\d+\.\d+\.\d+).*", file)[0]
                print(f'parsed app version {app_version}')
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.nsis.zip"
        elif file.endswith('.nsis.zip.sig'):
            if app_version is None:
                app_version = re.findall(r".*(\d+\.\d+\.\d+).*", file)[0]
                print(f'parsed app version {app_version}')
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.nsis.zip.sig"
        
        if filename is not None:
            copy_bundle_files(file_path, to_dir, filename)
    os.chdir('../scripts/')

def build_app_macos(target, build_platform):
    os.chdir('../')
    root_path = os.getcwd()
    os.chdir('./app/')
    app_path = os.getcwd()

    bundle_path = os.path.join(app_path, f'src-tauri/target/{target}/release/bundle')
    if os.path.exists(bundle_path):
        shutil.rmtree(bundle_path)

    execute(f'pnpm tauri build --target {target}')

    to_dir = os.path.join(root_path, 'bin', 'app', build_platform)

    dmg_bg_file = os.path.join(app_path, f'src-tauri/icons/macos/dmg-bg.png')
    dmg_icon_file = os.path.join(app_path, f'src-tauri/icons/macos/icon.icns')
    
    macos_path = os.path.join(bundle_path, 'macos')
    dmg_path = os.path.join(bundle_path, 'dmg')

    app_version = None
    for file in os.listdir(dmg_path):
        if file.endswith(".dmg"):
            app_version = re.findall(r".*(\d+\.\d+\.\d+).*", file)[0]
            print(f'parsed app version {app_version}')
            break

    app_file = None
    for file in os.listdir(macos_path):
        
        file_path = os.path.join(macos_path, file)

        if file.endswith('.app'):
            app_file = file_path
            continue

        filename = None

        if file.endswith('.app.tar.gz'):
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.app.tar.gz"
        elif file.endswith('.app.tar.gz.sig'):
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.app.tar.gz.sig"
        
        if filename is not None:
            copy_bundle_files(file_path, to_dir, filename)
    
    dmg_file = os.path.join(to_dir, f"{BUNDLE_NAME}-{app_version}-{build_platform}.dmg")

    if os.path.exists(dmg_file):
        os.unlink(dmg_file)

    create_dmg_cmd = f"""
create-dmg \\
--volname "{APP_NAME}" \\
--volicon "{dmg_icon_file}" \\
--background "{dmg_bg_file}" \\
--window-pos 200 200 \\
--window-size 900 600 \\
--icon-size 128 \\
--icon "{APP_NAME}" 232 338 \\
--hide-extension "{APP_NAME}" \\
--app-drop-link 688 338 \\
"{dmg_file}" \\
"{app_file}"
"""
    
    execute(create_dmg_cmd)
    print(f'create dmg: {dmg_file}')

def copy_bundle_files(from_file, to_dir, filename):
    
    if not os.path.exists(to_dir):
        os.makedirs(to_dir)
    
    to_file = os.path.join(to_dir, filename)
    shutil.copyfile(from_file, to_file)

    print(f'copied file {to_file}')