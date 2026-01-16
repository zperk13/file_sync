:: Thanks to https://dev.to/deciduously/prepare-your-rust-api-docs-for-github-pages-2n5i for the base of the script (I translated it to Windows and slightly tweaked it)

cargo doc --no-deps
rmdir /s ./docs
robocopy target/doc docs /s
echo|set /p="<meta http-equiv="refresh" content="0; url=file_sync/index.html">" > docs/index.html
