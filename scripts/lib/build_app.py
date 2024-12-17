# encoding=utf8

import subprocess
import platform
import os
import shutil
import re

APP_NAME = "Etcd Workbench"
BUNDLE_NAME = "etcd-workbench"
ENCODING = "gbk" if platform.system().lower() == 'windows' else 'utf-8'

def execute(command, encoding = ENCODING):
    print(f"calling: {command}")
    subprocess.call(command, shell=True, encoding=encoding)
    
def build_app_windows(target, build_platform):
    print("Building app source of windows...")
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
    print("Building app source of macos...")
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


def build_web(bin_name, skip_merge_jar = False):
    print("Building web source...")
    os.chdir('../')
    root_path = os.getcwd()
    
    os.chdir('web')
    web_path = os.getcwd()
    execute("nvm use 22")
    execute("pnpm install")
    execute("pnpm run build")

    print("Deleting server static files...")
    server_static_path = os.path.join(root_path, "server/src/main/resources/static")
    if os.path.exists(server_static_path):
        shutil.rmtree(server_static_path)

    shutil.copytree(os.path.join(web_path, 'dist'), server_static_path)

    os.chdir('../server')
    graldew_script = os.path.join(root_path, 'server', 'gradlew')
    execute(f'{graldew_script} clean')

    if skip_merge_jar:
        execute(f'{graldew_script} jar -PskipMerge=1', 'utf-8')
    else:
        execute(f'{graldew_script} jar', 'utf-8')
    
    os.chdir('../')
    to_dir = os.path.join(root_path, 'bin', 'web', bin_name)
    if os.path.exists(to_dir):
        shutil.rmtree(to_dir)

    shutil.copytree(os.path.join(root_path, 'server/build/libs'), to_dir)


def build_docs():
    print("Building docs...")
    os.chdir('../')
    root_path = os.getcwd()

    os.chdir('doc-source')
    doc_src_path = os.getcwd()

    # execute("nvm use 22")
    execute("pnpm install")
    execute("pnpm run docs:build")

    dist_path = os.path.join(doc_src_path, 'docs', '.vitepress', 'dist')

    os.chdir('../docs')
    docs_path = os.getcwd()
    for file in os.listdir(docs_path):
        if file!='etcd-workbench-update.json':
            path = os.path.join(docs_path, file)
            if os.path.isfile(path):
                os.unlink(path)
            else:
                shutil.rmtree(path)
    
    for file in os.listdir(dist_path):
        path = os.path.join(dist_path, file)
        if os.path.isfile(path):
            shutil.copy(path, docs_path)
        else: 
            shutil.copytree(path, os.path.join(docs_path, file))
    
    print("Copied files")
