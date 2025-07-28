
git pull

export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
export RUSTUP_VERSION=stable

cargo run 

cd quartz
npx quartz build
