//! PKCS#1 private key tests

use hex_literal::hex;
use pkcs1::{RsaPrivateKey, Version};

#[cfg(feature = "pem")]
use pkcs1::{der::Document, RsaPrivateKeyDocument};

/// RSA-2048 PKCS#1 private key encoded as ASN.1 DER.
///
/// Note: this key is extracted from the corresponding `rsa2048-priv.der`
/// example key in the `pkcs8` crate.
const RSA_2048_DER_EXAMPLE: &[u8] = include_bytes!("examples/rsa2048-priv.der");

/// RSA-4096 PKCS#1 private key encoded as ASN.1 DER
const RSA_4096_DER_EXAMPLE: &[u8] = include_bytes!("examples/rsa4096-priv.der");

/// RSA-2048 PKCS#1 private key with 3 primes encoded as ASN.1 DER
const RSA_2048_MULTI_PRIME_DER_EXAMPLE: &[u8] = include_bytes!("examples/rsa2048-priv-3prime.der");

/// RSA-2048 PKCS#1 private key encoded as PEM
#[cfg(feature = "pem")]
const RSA_2048_PEM_EXAMPLE: &str = include_str!("examples/rsa2048-priv.pem");

/// RSA-4096 PKCS#1 private key encoded as PEM
#[cfg(feature = "pem")]
const RSA_4096_PEM_EXAMPLE: &str = include_str!("examples/rsa4096-priv.pem");

#[test]
fn decode_rsa2048_der() {
    let key = RsaPrivateKey::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(key.version(), Version::TwoPrime);

    // Extracted using:
    // $ openssl asn1parse -in tests/examples/rsa2048-priv.pem
    assert_eq!(key.modulus.as_bytes(), hex!("B6C42C515F10A6AAF282C63EDBE24243A170F3FA2633BD4833637F47CA4F6F36E03A5D29EFC3191AC80F390D874B39E30F414FCEC1FCA0ED81E547EDC2CD382C76F61C9018973DB9FA537972A7C701F6B77E0982DFC15FC01927EE5E7CD94B4F599FF07013A7C8281BDF22DCBC9AD7CABB7C4311C982F58EDB7213AD4558B332266D743AED8192D1884CADB8B14739A8DADA66DC970806D9C7AC450CB13D0D7C575FB198534FC61BC41BC0F0574E0E0130C7BBBFBDFDC9F6A6E2E3E2AFF1CBEAC89BA57884528D55CFB08327A1E8C89F4E003CF2888E933241D9D695BCBBACDC90B44E3E095FA37058EA25B13F5E295CBEAC6DE838AB8C50AF61E298975B872F"));
    assert_eq!(key.public_exponent.as_bytes(), hex!("010001"));
    assert_eq!(key.private_exponent.as_bytes(), hex!("7ECC8362C0EDB0741164215E22F74AB9D91BA06900700CF63690E5114D8EE6BDCFBB2E3F9614692A677A083F168A5E52E5968E6407B9D97C6E0E4064F82DA0B758A14F17B9B7D41F5F48E28D6551704F56E69E7AA9FA630FC76428C06D25E455DCFC55B7AC2B4F76643FDED3FE15FF78ABB27E65ACC4AAD0BDF6DB27EF60A6910C5C4A085ED43275AB19C1D997A32C6EFFCE7DF2D1935F6E601EEDE161A12B5CC27CA21F81D2C99C3D1EA08E90E3053AB09BEFA724DEF0D0C3A3C1E9740C0D9F76126A149EC0AA7D8078205484254D951DB07C4CF91FB6454C096588FD5924DBABEB359CA2025268D004F9D66EB3D6F7ADC1139BAD40F16DDE639E11647376C1"));
    assert_eq!(key.prime1.as_bytes(), hex!("DCC061242D4E92AFAEE72AC513CA65B9F77036F9BD7E0E6E61461A7EF7654225EC153C7E5C31A6157A6E5A13FF6E178E8758C1CB33D9D6BBE3179EF18998E422ECDCBED78F4ECFDBE5F4FCD8AEC2C9D0DC86473CA9BD16D9D238D21FB5DDEFBEB143CA61D0BD6AA8D91F33A097790E9640DBC91085DC5F26343BA3138F6B2D67"));
    assert_eq!(key.prime2.as_bytes(), hex!("D3F314757E40E954836F92BE24236AF2F0DA04A34653C180AF67E960086D93FDE65CB23EFD9D09374762F5981E361849AF68CDD75394FF6A4E06EB69B209E4228DB2DFA70E40F7F9750A528176647B788D0E5777A2CB8B22E3CD267FF70B4F3B02D3AAFB0E18C590A564B03188B0AA5FC48156B07622214243BD1227EFA7F2F9"));
    assert_eq!(key.exponent1.as_bytes(), hex!("CE68B7AC1B0D100D636E55488753C5C09843FDB390E2705DF7689457C9BD8D9765E30978617E2EFC8048F4C324206DB86087B654E97BB3D464E7EE3F8CD83FE10436F7DF18E9A963C4E64911D67EDE34042F2E26E3D3A1AD346ADAD6B9B7F67708CB094E62DEE9FF4D5D6669AF988AF2255D1CE8ED317C6A7D8691DA354D12DB"));
    assert_eq!(key.exponent2.as_bytes(), hex!("25F6E5944220286B4DFBBF4235C0EE5843D2198091895120D6CA7B200B826D3ECE738E2E00498FAC0A2A6CA969C7F0C3CA1AB0BC40297132BE7538D7BEDF4CB0EFC6B98EF7DBA54F56AA99AABCE534C49C27947D4678C51C63C78C7CE1687231B4C8EB587AE6EF0480CBAF4FC0173CFD587A7E67AF515FB9B9DE751118397229"));
    assert_eq!(key.coefficient.as_bytes(), hex!("31995406D406207CADEAEA35B38D040C5F8A9A1AE0827E9ED06B153D83B6821935B4B36A82BE9D56C791B58C27271A5793D53A1D657C08997960B1433E5171987F452F144A7C72306D63E1D3FFC0B71B75AB08F2E45A482E988451CBE478E12EB228D07456C924B66F6CED048D853F533E31A68614F1C3CE6D8EC9983CE72AF7"));
    assert!(key.other_prime_infos.is_none());
}

#[test]
fn decode_rsa4096_der() {
    let key = RsaPrivateKey::try_from(RSA_4096_DER_EXAMPLE).unwrap();
    assert_eq!(key.version(), Version::TwoPrime);

    // Extracted using:
    // $ openssl asn1parse -in tests/examples/rsa4096-priv.pem
    assert_eq!(key.modulus.as_bytes(), hex!("A7A74572811EA2617E49E85BD730DDE30F103F7D88EE3F765E540D3DD993BBB0BA140002859D0B40897436637F58B828EA74DF8321634077F99D4AA2D54CA375852EF597661D3713CE1EF3B4FD6A8E220238E467668A2C7EE3861D2212AE6A1EBDDFA88B62DF10F6BCF79EFF4AC298FB2563DF1B8764381AF9B1FB0CCD085E026B0AD9F6721A235177D0396B48754AD4A75242250A873BF2F6E7EE3C75DD613E365BA4F3210A6CC66B90A2FA3F762CA6884087B6BF8161EB144819F0F572F21F6C8E273E70D45A365B8B2819CE734613CC23B01329A17901F17078403861F54C52A051E2A58C75C2D9D80091BB9808A106C1F7ECB4034E15058BEEC725C5F919D62EAA234B62628D346C60BB919E70851DAB38571E6F0ED7634129F994EA368FEE7373DFDEC04445EBCA47FA20ED1540A860C948BABC98DA591CA1DE2E2E25540EF9B7CB353F60213B814A45D359EFA9B811EEFF08C65993BF8A85C2BFEAAA7ED5E6B43E18AE604464CE5F96150136E7D09F8B24FAD43D7870118CFA7BC24875506EBBC321B977E0861AEA50128620121F0B394A9CDD0A42411A1350C0770D975D71B00A90436240C967A0C3A5C20A0F6DE77F3F2CAFDA94ED0143C1F6E34F73E0CAC279EEEB7C637723A2B026C82802E1A4AEBAA8846DF98E7919498773E0D4F319956F4DE3AAD00EFB9A147D66B3AC1A01D35B2CFB48D400B0E7A80DC97551"));
    assert_eq!(key.public_exponent.as_bytes(), hex!("010001"));
    assert_eq!(key.private_exponent.as_bytes(), hex!("9FE3097B2322B90FAB6606C017A095EBE640C39C100BCEE02F238FA14DAFF38E9E57568F1127ED4436126B904631B127EC395BB3EE127EB82C88D2562A7FB55FED8D1450B7E4E2D2F37F5742636FCC6F289963522D5B5706082CADFA01C0EE99B4D0E9274D3A992E06974CBE01694686356962AC1959FD9BD447E5B9968C0543DF1BF134742AF345CDB2FA1F9371B0D4CF61C68D16D653D8E999D4FD3A16CF978A35AA40E860CDCE09655DD8B4CF19D4141B1E92AD5E51A8E4A5C27FA745611D90E49D0E9282222AB6F126643E1C77578816FCE3B98F321D2549F294A470DF8453446BF36F985DF25ED8FDE9FDF3073FB27727DF48E9E1FC7056BC78965090B7850126406462C8253051EF84E34EE3C3CEB8F96C658C38BE45558D2F64E29D223350555FC1EFA28EC1F4AFB5BA4080F09A86CDC3538C1AD7C972E6D7A3612E6845BA9AFBDF19F09060D1A779DE9635E2D2F8E0C510BA24C6C44B30C9BDFAF85BE917AEC5D43AFAB1AA3ADD33CC83DA93CAC69218F6A36EB47F199D5424C95FD9ED7B1E8BE2AEAA6433B227241316C20EE792650CEB48BFD634446B19D286B4EA1722498DA1A36973210EC3824751A5808D9AAEF59C449E19A5077CFECA126BD9A8DD4996561D4E27B3609FF82C5B1B21E627845D44961B33B875D5C4FA9FF357EF6BE3364969E1337C91B29A07B9A913CDE40CE2D5530C900E73751685E65431"));
    assert_eq!(key.prime1.as_bytes(), hex!("D0213A79425B665B719118448893EC3275600F63DBF85B77F4E8E99EF302F6E82596048F6DCA772DE6BBF1124DB84B0AFE61B03A8604AB0079ED53F3304797AD01B38C44FE27A5A45E378483A804B56A4A967F48F01A866E721E67E4C9A1048AF68927FAA43D6A85D93E7BF7074DBA797563FCABE12309B76653C6DB614DC231CC556D9F25AC4841A02D31CDF3015B212307F9D0C79FEB5D3956CE53CC8FA1651BE60761F19F74672489EAF9F215409F39956E77A82183F1F72BB2FEDDF1B9FBFC4AD89EA445809DDBD5BD595277990C0BE9366FBB2ECF7B057CC1C3DC8FB77BF8456D07BBC95B3C1815F48E62B81468C3D4D9D96C0F48DAB04993BE8D91EDE5"));
    assert_eq!(key.prime2.as_bytes(), hex!("CE36C6810522ABE5D6465F36EB137DA3B9EA4A5F1D27C6614729EB8E5E2E5CB88E3EF1A473A21944B66557B3DC2CE462E4BF3446CB4990037E5672B1705CBAE81B65BAF967A266DC18EFE80F4DBBFE1A59063205CE2943CADF421CCE74AF7063FD1A83AF3C39AF84525F59BDC1FF54815F52AFD1E8D4862B2C3654F6CFA83DC08E2A9D52B9F833C646AF7694467DFC5F7D7AD7B441895FCB7FFBED526324B0154A15823F5107C89548EDDCB61DA5308C6CC834D4A0C16DFA6CA1D67B61A65677EB1719CD125D0EF0DB8802FB76CFC17577BCB2510AE294E1BF8A9173A2B85C16A6B508C98F2D770B7F3DE48D9E720C53E263680B57E7109410015745570652FD"));
    assert_eq!(key.exponent1.as_bytes(), hex!("748B46CD03E55E69B22C476488FE1BF31D5ACF0361F7AE707B89B8D832C7E42E966D6CDC4BE465DC2429F5920447406E4587BA40EB2ECDFA944BDB08806E7676804F642A760F096803021F88019BB16275A5D45CA9669104638EB72A9BE538400051493BC6A04577F1F055463CA6BFD6A76F77DB5F54596A83384250322A72A5A3FFEA4485B9F5341A57745E18C7179A749D50BC222C60857148347D243D016936B816463820CBF3BDB825061512E57EC3A5F397B9641B187109DD4F6E449F9A84E9FC66C921CA259B2612C363B468D5200E5557377FBCDAEC75B1A2D56CFC97C4AC4BA35AFA23C680CE3A8548AE3F6F72C94BBBBE10C900FC5A170B4B06FE29"));
    assert_eq!(key.exponent2.as_bytes(), hex!("75CA4E0B069EEE67C3C4C0C082F8C82C8C96EAD277B9EF94436D0B936FF2B59DEA0AC446B6926232A0A934B6954EC34A45F57DEBEE54DFC14F1A1C3B84BE43392FE5252F2F6651B0E941A8618D7A93C4031409E0CD093F2313F214B84D68A51F48452BF11DCAA99A40DF1C48CB1688F3B93A6719D510086F82BAAA3FAD1021EDEA872704491C209EE26379AD6AB2AE44F14D09077AE3F8672A7D01EBAC9C19449FE3B7596974B3BBAA43CC6DEE731C4F2A18162D5A8202CB27E02DBE9E61C0449171C9981D2430D39DE28C298D8D50A943B2F27C5E665CBAB2897959FF19A5E87E632C58CDC31F9BDE9BC100AFFFDF50CF210F1E63A0A6149D2BD6E8D1B3D815"));
    assert_eq!(key.coefficient.as_bytes(), hex!("8C557C4835E8B4066606A5DC255DBC3EB5F1586C7328988C8E242403F9575EFB0D28012C0634B7241768AD722049382906D635AFCB1D2A05BDFFC805FE670389B7FA90D65F94AFBC2008D9BC0D3CCEE425DFC72402050603C9F579EE8510ECC693AC6DD32836B6E15E3DC520366A1BA4AFC91D3E48A4D6724CA42443F4803E3B12634B0438572D8E13BC72F1A82853B8D9FCA6199D7A90AB64D9E79F793AC287F9A19011082A5268F209332D54D98D6E19030907FD9E6B82EF124677BA08AD6797A442213D097A351C50A08928DD732D6B29A6CF6706DAF5F20F78F87170FB279403C5221B5E5C70C2BAA0E69A72E0D1F3267DBD8C9BDDFF28AFA92F37C98D36"));
    assert!(key.other_prime_infos.is_none());
}

#[cfg(not(feature = "alloc"))]
#[test]
fn decode_rsa2048_multi_prime_der() {
    // Multi-prime RSA keys are unsupported when the alloc feature is disabled
    assert!(RsaPrivateKey::try_from(RSA_2048_MULTI_PRIME_DER_EXAMPLE).is_err());
}

#[cfg(feature = "alloc")]
#[test]
fn decode_rsa2048_multi_prime_der() {
    let key = RsaPrivateKey::try_from(RSA_2048_MULTI_PRIME_DER_EXAMPLE).unwrap();
    assert_eq!(key.version(), Version::Multi);

    // Extracted using:
    // $ openssl asn1parse -in tests/examples/rsa2048-priv-3prime.pem
    assert_eq!(key.modulus.as_bytes(), hex!("E8BF46480D9144E93D0E7971554F7EF45FB25B83A1069416DB8EC7C654EB489C695A3274EE4B5F3A08E35BA6D1FF2EC487CE544641EA0C0994CC554D2201308E3AC6491326183F477D30F165001800B38771BF96F69DE6D461A6E2F88057104FCDD6C6A1B69E7F95A27DAC0CBAE39CC1F9264C09DFEEA68477482C2816133012BA29A0D617FA6C6B70FAC3BEDCB020CB1F6EA3375163988E7F8C9474B5462C24D2B56BC03F9ED002E43C764B1E7B05C31EC7D8E91582CEC0942F396DAB94F3162D1AAA041DA99CEE90B81FF408BB75B7B233B19D34E504E258C1D774706E4C5FC2B11F97C9E4CEEAC345F8C57C3D3702F73FC98F7EED688812BE9B560618C245"));
    assert_eq!(key.public_exponent.as_bytes(), hex!("010001"));
    assert_eq!(key.private_exponent.as_bytes(), hex!("51ADA6656CD579207CFBD2649272B673DE0D828E1BF96A08E77E20DF9A3783A0D85BFDEF091D4C4ADA89A74550D6C3BBD688F30C40DF78DFF7E7095C6B3D8DA3AC3E9FB067A304B9FAD62D30ABAAC0BC40210C025759A374034864308AF6E444C6BEEA4FEA12DB78A9705A0F0C8B732B3FD733C5877A871ECC58FAB25D4236D0C91F207D6FC79ED5568658623984EF992C85EA7BFFC7F8C18368E644289198B64CF368C0FDEFFA7E24B65D895B701F8AF70AEAB0BB7D2E14C4354E15BE4723EC885DF146D7B0C6D2844C14DF639DDA9C7933FFFBD5AE2308B2A9CE7E6C52E3766106B8FEF8D593B88491BCED68632045E0E2FB889A58ED2FF28B6E5E2082BB01"));
    assert_eq!(key.prime1.as_bytes(), hex!("07EA41E3B0B6FA6337D8D254B21F06C237FDAA6442B3B2756A479B27273CECCFBB5F472AA5D941E11DA1AAEB176450FC044EA639EF6E6B77B530D98A43ACF2D39571714EED516BBF43A7079E640CA46AE640CCAE46B5"));
    assert_eq!(key.prime2.as_bytes(), hex!("077E9210399453F1F9D6AD862B4D83EE8651F92354BC4F22B1A5376ED36FE80DCFDDF73CEEB326972265B5CAC21E2F5FF22F09E6EA0AF358540004D6D1EF3A82BE214D206B25F98F0F3BAF8CD4F25D7169ACC99809A9"));
    assert_eq!(key.exponent1.as_bytes(), hex!("02362298C27C2D73615C345033C0557C1876C88FE0CF227289F25DD84FAF471F3764049756E55FF1CCFCA9ABABCA7C921D85F80DC1E72121BE3F5AD8B5D5F1B6CA4F7AF0298091514C4EB3E33E6305E1545A109634ED"));
    assert_eq!(key.exponent2.as_bytes(), hex!("830FFD55C1A13E6D7FD1D8D18A7911C9513E40C0A02093D128E53187F62417157527579D42D5C70B4D816EB976106C7081F01E3B2CC654E9601AF485E0DED161A5AABC2535B1E7A8F5BDD75410BE7B69E9A8D7DF09"));
    assert_eq!(key.coefficient.as_bytes(), hex!("03B54C52199604D9A920B1B7AB95F0B474BA46796CC46C3E63C483499071BFADC0D27321EBD53E09271F28646B7590C97102F9E458A93B3A024A03565BCAFFA15F73C05BBF9653BE23224A1E35524A9895641C76FD2E"));

    let other_prime_infos = key.other_prime_infos.unwrap();
    assert_eq!(other_prime_infos.len(), 1);
    assert_eq!(other_prime_infos[0].prime.as_bytes(), hex!("03EC75532FB0D2CA142632618EA698548D8489FE30A868D637A1D53895C79F16A63ADB60DB217DDC0D95694A3D95D029D1A9A2DF548ED105B2C6A84855739EDBC3CE82906500241D518A3172F8D470FADD4E1DFE5969"));
    assert_eq!(other_prime_infos[0].exponent.as_bytes(), hex!("03A11508A3E52FA4412CEF8EF34EBF39FE48690758647DCC1F5B2E890F69BC8A4BA9C73F78912B047F00038AEB1A069897D90BD0FD3AB8B6479D9F0C8115D80BB8BAEC63B9387F2F2B3BE2EF509FD7FD02F47DA3C579"));
    assert_eq!(other_prime_infos[0].coefficient.as_bytes(), hex!("39EA226CABFB317E41A5593B9168D1A0124993B45D9CD14A22BD1557CDCB43D28024AC26ED2C8530B53E9B93A878F428807C5282EBB811399F913017CDF2149013D80CDF73F609D6C692475EB7A123D0E93E6A60FC"));
}

#[cfg(feature = "pem")]
#[test]
fn decode_rsa_2048_pem() {
    let pkcs1_doc: RsaPrivateKeyDocument = RSA_2048_PEM_EXAMPLE.parse().unwrap();
    assert_eq!(pkcs1_doc.as_ref(), RSA_2048_DER_EXAMPLE);

    // Ensure `RsaPrivateKeyDocument` parses successfully
    let pk = RsaPrivateKey::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    assert_eq!(pkcs1_doc.decode().modulus.as_bytes(), pk.modulus.as_bytes());
}

#[cfg(feature = "pem")]
#[test]
fn decode_rsa_4096_pem() {
    let pkcs1_doc: RsaPrivateKeyDocument = RSA_4096_PEM_EXAMPLE.parse().unwrap();
    assert_eq!(pkcs1_doc.as_ref(), RSA_4096_DER_EXAMPLE);

    // Ensure `RsaPrivateKeyDocument` parses successfully
    let pk = RsaPrivateKey::try_from(RSA_4096_DER_EXAMPLE).unwrap();
    assert_eq!(pkcs1_doc.decode().modulus.as_bytes(), pk.modulus.as_bytes());
}

#[test]
fn private_key_to_public_key() {
    let private_key = RsaPrivateKey::try_from(RSA_2048_DER_EXAMPLE).unwrap();
    let public_key = private_key.public_key();

    // Extracted using:
    // $ openssl asn1parse -in tests/examples/rsa2048-pub.pem
    assert_eq!(public_key.modulus.as_bytes(), hex!("B6C42C515F10A6AAF282C63EDBE24243A170F3FA2633BD4833637F47CA4F6F36E03A5D29EFC3191AC80F390D874B39E30F414FCEC1FCA0ED81E547EDC2CD382C76F61C9018973DB9FA537972A7C701F6B77E0982DFC15FC01927EE5E7CD94B4F599FF07013A7C8281BDF22DCBC9AD7CABB7C4311C982F58EDB7213AD4558B332266D743AED8192D1884CADB8B14739A8DADA66DC970806D9C7AC450CB13D0D7C575FB198534FC61BC41BC0F0574E0E0130C7BBBFBDFDC9F6A6E2E3E2AFF1CBEAC89BA57884528D55CFB08327A1E8C89F4E003CF2888E933241D9D695BCBBACDC90B44E3E095FA37058EA25B13F5E295CBEAC6DE838AB8C50AF61E298975B872F"));
    assert_eq!(public_key.public_exponent.as_bytes(), hex!("010001"));
}
