
[![Tests](https://github.com/mverleg/file_endec/workflows/Test%20file_endec/badge.svg)](https://github.com/mverleg/file_endec/actions)

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
* Hiding of file name, size and metadata.

In Docker
-------------------------------

Run the encryptor with Docker::

    docker run --rm -it -v "$(pwd):/data" mverleg/file-endec /fileenc file.txt

You can mount any directory in which you want to encrypt files; the above example uses the current directory `$(pwd)`. Use `/filedec` instead of `/fileenc` for decryption.

To build the image yourself (instead of downloading from Dockerhub), clone the Github project and run::

    docker build -t mverleg/file-endec .

This will also run the tests and lints, to verify that your version is okay.

Options
-------------------------------

Use `fileenc --help` and `filedec --help` to see CLI arguments. For `fileenc`:

    USAGE:
        fileenc [FLAGS] [OPTIONS] <FILES>...
    
    FLAGS:
            --accept-weak-key    Suppress warning if the encryption key is not strong.
        -v, --debug              Show debug information, especially on errors.
        -d, --delete-input       Delete unencrypted input files after successful encryption (overwrites garbage before delete).
            --dry-run            Test encryption, but do not save encrypted files (nor delete input, if --delete-input).
        -s, --fast               Use good instead of great encryption for a significant speedup.
        -h, --help               Prints help information
            --hide-meta          Hide name, timestamp and permissions.
            --hide-size          Hide the exact compressed file size, by padding it to the next power of two.
        -f, --overwrite          Overwrite output files if they exist.
        -q, --quiet              Do not show progress or other non-critical output.
        -V, --version            Prints version information
    
    OPTIONS:
        -k, --key <key-source>
                Where to get the key; one of 'pass:$password', 'env:$var_name', 'file:$path', 'ask', 'ask-once', 'pipe' [default: ask]
        -o, --output-dir <output-dir>
                Alternative output directory. If not given, output is saved alongside input.
            --output-extension <output-extension>    Extension added to encrypted files. [default: .enc]
    
    ARGS:
        <FILES>...    One or more paths to input files (absolute or relative)

The `--fast` mode uses only one hash algorithm one encryption algorithm (argon2i and aes256), and no key stretching; this makes it about 10 times faster.

Keep in mind
-------------------------------

*While this mostly relies on established hashing and encryption algorithms, there are no security guarantees, and the author is not a professional security expert. Use at your own risk.*

* Encrypting the same file twice will give different results, which is needed for semantically security. This may be suboptimal for version control.
* When hashing multiple files, they share the same salt. This choice was made because stretching takes long, and because if one key were to be found somehow, it would work for all files regardless of salts.
