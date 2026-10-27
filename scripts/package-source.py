"""Publish only allowlisted project sources. Never walk sibling projects."""
import hashlib,json,pathlib,re,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
ROOT_FILES=('README.md','README.tr.md','LICENSE','SECURITY.md','CONTRIBUTING.md','THIRD_PARTY_NOTICES.md','CHANGELOG.md','PLAN.tr.md','Cargo.toml','Cargo.lock','rust-toolchain.toml','package.json','package-lock.json','tsconfig.json','vite.config.ts','index.html','.gitignore','.gitattributes','.prettierrc.json')
DIRS=('src','src-tauri','crates','tests','fixtures','scripts','docs','.github','tools')
ALLOWED={'.md','.rs','.toml','.json','.lock','.ts','.tsx','.mjs','.py','.ps1','.css','.html','.svg','.png','.ico','.yml','.yaml','.txt','.csv'}
SECRET=[re.compile(r'sk-[A-Za-z0-9_-]{25,}'),re.compile(r'gh[pousr]_[A-Za-z0-9]{30,}'),re.compile(r'github_pat_[A-Za-z0-9_]{30,}'),re.compile(r'-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----')]
files={}
paths=[ROOT/name for name in ROOT_FILES]
for folder in DIRS:
    for path in (ROOT/folder).rglob('*'):
        if path.is_relative_to(ROOT/'tools/notices'):continue
        if path.is_relative_to(ROOT/'src-tauri/gen'):continue
        if any(p in ('target','node_modules','artifacts','.git','__pycache__') for p in path.relative_to(ROOT).parts):continue
        if path.suffix=='.exe':continue
        if path.is_file():paths.append(path)
for path in sorted(set(paths)):
    assert path.is_file(),str(path)
    assert not path.is_symlink() and path.resolve().is_relative_to(ROOT)
    assert path.name in ROOT_FILES or path.suffix in ALLOWED,str(path)
    name=path.relative_to(ROOT).as_posix()
    if path.suffix=='.csv':assert name in ('fixtures/synthetic-baseline.csv','fixtures/synthetic-candidate.csv')
    data=path.read_bytes()
    if path.suffix not in ('.png','.ico'):
        data=data.replace(b'\r\n',b'\n')
        text=data.decode('utf8')
        assert not any(pattern.search(text) for pattern in SECRET),'Possible secret: '+name
    files[name]=data
assert len(files)<500 and sum(map(len,files.values()))<12*1024*1024
assert {'Cargo.lock','package-lock.json','tools/presentmon.lock.json','tools/presentmon/LICENSE.txt','src-tauri/icons/icon.ico'}.issubset(files)
out=ROOT/'artifacts/release';out.mkdir(parents=True,exist_ok=True)
manifest={name:hashlib.sha256(data).hexdigest() for name,data in files.items()}
version=json.loads((ROOT/'package.json').read_text(encoding='utf8'))['version']
assert re.fullmatch(r'\d+\.\d+\.\d+-alpha\.\d+',version)
assert json.loads((ROOT/'src-tauri/tauri.conf.json').read_text(encoding='utf8'))['version']==version
archive=out/f'nexora-framelab-{version}-source.zip'
with zipfile.ZipFile(archive,'w',zipfile.ZIP_DEFLATED,compresslevel=9) as package:
    for name,data in {**files,'SOURCE_MANIFEST.json':(json.dumps(manifest,indent=2)+'\n').encode()}.items():
        info=zipfile.ZipInfo('nexora-framelab/'+name,(2026,10,27,0,0,0));info.compress_type=zipfile.ZIP_DEFLATED;info.external_attr=0o100644<<16;package.writestr(info,data)
sha=hashlib.sha256(archive.read_bytes()).hexdigest()
(out/'SOURCE-SHA256SUMS.txt').write_text(sha+'  '+archive.name+'\n',encoding='ascii')
print(json.dumps({'files':len(files),'bytes':archive.stat().st_size,'sha256':sha}))
