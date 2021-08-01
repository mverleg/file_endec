
# File format encrypt/decrypt

## Blocks

* **Public header** (plain text)

  * Version
  * Salt
  * Data checksum (only before v1.1)
  * Options:

    * Fast
    * HideMeta
    * PadSize

  * Private header:
  
    * Length (bytes)
    * Checksum

* **Private header** (v1.1+, encrypted)

  * Filename
  * Permissions (optional)
  * Created (ns) (optional)
  * Changed (ns) (optional)
  * Accessed (ns) (optional)
  * Data info:

    * Length (bytes)
    * Checksum

  * Pepper
  * Header padding length

* **Data**

* **Padding**

## Layers




* Public header
     
  * 

* Private header (v1.1+) & Data

  * Compressed with Brotli
  * Encrypted with Aes256 and/or Twofish

* Padding with random data


