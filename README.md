
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

How to use
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
