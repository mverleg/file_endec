
[![Tests](https://github.com/mverleg/file_shred/workflows/Test%20file_endec/badge.svg)](https://github.com/mverleg/file_endec/actions)

[![Dependencies](https://deps.rs/repo/github/mverleg/file_endec/status.svg)](https://deps.rs/repo/github/mverleg/file_endec)

File encrypt/decrypt
===============================

Command line utility that encrypts and decrypts files.

Functionality
-------------------------------

* Encryption and decryption using established algorithms.
* Compression.
* Key stretching.
* Salts.
* Checksums.
* Backward-compatibility.
* Pass keys by prompt, argument, environment, file or pipe.
* Warnings for weak keys.
* Shredding of deleted files.

In Docker
-------------------------------

Run the encrypter with Docker::

    docker run --rm -it -v "$(pwd):/data" file_endec -- encrypt file.txt

You can mount any directory in which you want to encrypt files; the above example uses the current directory `$(pwd)`. Use `decrypt` instead of `encrypt` for decryption.

To build the image yourself (instead of downloading from Dockerhub), clone the Github project and run::

    docker build -t file_endec .

This will also run the tests and lints, to verify that your version is okay.

Options
-------------------------------

Use `fileenc --help` and `filedec --help` to see CLI arguments. For `fileenc`:

    USAGE:
        fileenc [FLAGS] [OPTIONS] <FILES>...

    FLAGS:
            --accept-weak-key    Suppress warning if the encryption key is not strong.
        -v, --debug              Show debug information, especially on errors.
        -d, --delete-input       Delete unencrypted input files after successful encryption (overwrites garbage before
                                 delete).
            --dry-run            Test encryption, but do not save encrypted files (nor delete input, if --delete-input).
        -h, --help               Prints help information
        -f, --overwrite          Overwrite output files if they exist.
        -q, --quiet              Do not show progress or other non-critical output.
        -V, --version            Prints version information

    OPTIONS:
        -k, --key <key-source>
                Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path', 'ask', 'askonce', 'pipe'
                [default: ask]
        -o, --output-dir <output-dir>
                Alternative output directory. If not given, output is saved alongside input.

            --output-extension <output-extension>    Extension added to encrypted files. [default: .enc]

    ARGS:
        <FILES>...    One or more paths to input files (absolute or relative)

Keep in mind
-------------------------------

*While this mostly relies on established hashing and encryption algorithms, there are no security guarantees, and the author is not a professional security expert. Use at your own risk.*

* Encrypting the same file twice will give different results, which is needed for semantically security. This may be suboptimal for version control.
* When hashing multiple files, they share the same salt. This choice was made because stretching takes long, and because if one key were to be found somehow, it would work for all files regardless of salts.
