# Running
If you're using WSL2 (which you should be), use the win.sh script. It will create a Windows executable and run it. The linux build runs as well using WSLg, but it has issues with cursor tracking.
You'll need the following:
```sh
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64 
```