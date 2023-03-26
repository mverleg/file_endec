use ::aes::Aes256;
use ::twofish::Twofish;

type Aes256Cbc = Cbc<Aes256, Iso7816>;
type TwofishCbc = Cbc<Twofish, Iso7816>;

pub mod encrypt;

pub mod decrypt;
