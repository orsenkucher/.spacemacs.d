### Enable ::std library completion with rust-analyzer on termux

To enable `rust-analyzer` completion you should provide it with rust sources.
Usually rustup components add rust-src will work, but on termux there is little trouble
with it.  
So fist you need to download sources from [forge.rust](https://forge.rust-lang.org/infra/other-installation-methods.html) 
in `Source code` section.  
tar xf rustc-???.gz - to unzip
move this folder as $(rustc --print sysroot)/lib/rustlib/src/rust   
for me it was `/data/data/com.termux/files/usr/opt/rust-nightly/lib/rustlib/src/rust/src`  
make symlink to this sources into `/data/data/com.termux/files/home/.rustup/toolchains/armv7-linux-androideabi/lib/rustlib/src/rust`  

```bash
ln -s /data/data/com.termux/files/usr/opt/rust-nightly/lib/rustlib/src/rust /data/data/com.termux/files/home/.rustup/toolchains/armv7-linux-androideabi/lib/rustlib/src/rust
```

it's save to `rm` symlinks
