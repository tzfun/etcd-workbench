# encoding=utf8

import subprocess
import platform
import os
import shutil
import re
import json
from datetime import datetime, timezone, timedelta
import time
from pathlib import Path
from dotenv import load_dotenv

APP_NAME = "Etcd Workbench"
BUNDLE_NAME = "etcd-workbench"
ENCODING = "gbk" if platform.system().lower() == 'windows' else 'utf-8'

def execute(command, encoding = ENCODING):
    print(f"calling: {command}")
    subprocess.call(command, shell=True, encoding=encoding)

def check_env():
    # 加载 .env 文件中的环境变量
    load_dotenv()
    if os.getenv('TAURI_PRIVATE_KEY') is None:
        raise Exception("Missing env: TAURI_PRIVATE_KEY")
    if os.getenv('TAURI_KEY_PASSWORD') is None:
        raise Exception("Missing env: TAURI_KEY_PASSWORD")

def copy_bundle_files(from_file, to_dir, filename):
    if not os.path.exists(to_dir):
        os.makedirs(to_dir)
    to_file = os.path.join(to_dir, filename)

    if os.path.exists(to_file):
        os.remove(to_file)
    
    shutil.copyfile(from_file, to_file)

    print(f'copied file {to_file}')

def to_rfc3339(dt=None, include_ms=False):
    """
    将 datetime 对象转换为 RFC 3339 格式字符串
    
    Args:
        dt: datetime 对象，默认为当前时间
        include_ms: 是否包含毫秒
    
    Returns:
        RFC 3339 格式的时间字符串
    """
    if dt is None:
        dt = datetime.now()
    
    # 处理时区信息
    if dt.tzinfo is None:
        # 无时区信息，使用本地时区
        timestamp = dt.timestamp()
        timezone_offset = time.localtime(timestamp).tm_gmtoff
        offset = timedelta(seconds=timezone_offset)
        dt = dt.replace(tzinfo=timezone(offset))
    
    # 格式化时间
    timespec = 'milliseconds' if include_ms else 'seconds'
    return dt.isoformat(timespec=timespec)

def update_updater_json(app_version, target_alias, exe_filename, sig_filepath):
    sig = None
    with open(sig_filepath, 'r', encoding='utf-8') as file:
        sig = file.read()

    github_updater_file = os.path.join(os.getcwd(), '../docs/etcd-workbench-update.json')
    github_download_url = f"https://github.com/tzfun/etcd-workbench/releases/download/App-{app_version}/{exe_filename}"
    update_updater_file(app_version, target_alias, sig, github_updater_file, github_download_url)

    gitee_updater_file = os.path.join(os.getcwd(), '../docs/etcd-workbench-update-gitee.json')
    gitee_download_url = f"https://gitee.com/tzfun/etcd-workbench/releases/download/App-{app_version}/{exe_filename}"
    update_updater_file(app_version, target_alias, sig, gitee_updater_file, gitee_download_url)

def update_updater_file(app_version, target_alias, sig, updater_filepath, download_url):
    updater_json = None
    with open(updater_filepath, 'r', encoding='utf-8') as file:
        updater_json = json.load(file)
    
    updater_json['version'] = app_version
    updater_json['pub_date'] = to_rfc3339()
    updater_json['platforms'][target_alias] = {
        'signature': sig,
        'url': download_url
    }

    with open(updater_filepath, 'w', encoding='utf-8') as file:
        file.write(json.dumps(updater_json, indent=4))
    print(f'updated updater file: {updater_filepath}')

def build_app_windows(target, build_platform):
    check_env()
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
    exe_filename = None
    sig_filename = None
    for file in os.listdir(nsis_path):
        file_path = os.path.join(nsis_path, file)
        filename = None
        if file.endswith('.exe'):
            if app_version is None:
                app_version = re.findall(r".*(\d+\.\d+\.\d+).*", file)[0]
                print(f'parsed app version {app_version}')
            filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.exe"
            exe_filename = filename
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
            sig_filename = filename
        
        if filename is not None:
            copy_bundle_files(file_path, to_dir, filename)
    os.chdir('../scripts/')
    if sig_filename is not None and exe_filename is not None:
        update_updater_json(app_version, build_platform, exe_filename, os.path.join(to_dir, sig_filename))

def build_app_macos(target, build_platform):
    check_env()
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
    exe_filename = None
    sig_filename = None
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
            sig_filename = filename
        
        if filename is not None:
            copy_bundle_files(file_path, to_dir, filename)
    
    exe_filename = f"{BUNDLE_NAME}-{app_version}-{build_platform}.dmg"
    dmg_file = os.path.join(to_dir, exe_filename)

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

    if sig_filename is not None and exe_filename is not None:
        update_updater_json(app_version, build_platform.replace('macos', 'darwin'), exe_filename, os.path.join(to_dir, sig_filename))

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
        if file.startswith('etcd-workbench-update'):
            continue
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
