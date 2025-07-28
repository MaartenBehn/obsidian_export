
git pull

rustup default stable
export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/

cargo run 

cd quartz
npx quartz build
