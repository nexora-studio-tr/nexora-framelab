"""Render the repository-native N mark; no external images or dependencies."""
import pathlib,struct,zlib,binascii
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'src-tauri/icons';OUT.mkdir(parents=True,exist_ok=True)
def chunk(name,data):return struct.pack('>I',len(data))+name+data+struct.pack('>I',binascii.crc32(name+data)&0xffffffff)
def png(size):
    rows=[]
    for y in range(size):
        row=bytearray()
        for x in range(size):
            a=x/size;b=y/size
            n=(.21<a<.34 and .21<b<.79) or (.66<a<.79 and .21<b<.79) or (.27<a<.73 and abs(b-(a*1.12-.06))<.085)
            row.extend((220,145,91,255) if n else (21,40,44,255))
        rows.append(b'\0'+row)
    return b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',size,size,8,6,0,0,0))+chunk(b'IDAT',zlib.compress(b''.join(rows),9))+chunk(b'IEND',b'')
for size,name in ((32,'32x32.png'),(128,'128x128.png'),(256,'128x128@2x.png')):(OUT/name).write_bytes(png(size))
data=png(256)
(OUT/'icon.ico').write_bytes(struct.pack('<HHH',0,1,1)+struct.pack('<BBBBHHII',0,0,0,0,1,32,len(data),22)+data)
print('Repository-native application icons generated.')
