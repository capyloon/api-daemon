const CHECK_RESPONSE_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<GOTU>
    <UPDATE_DESC>##version##</UPDATE_DESC><ENCODING_ERROR>0</ENCODING_ERROR><CUREF>%curef%</CUREF>
    <VERSION>
        <TYPE>2</TYPE><FV>%version%</FV><TV>%target%</TV><SVN>%target%</SVN>
        <RELEASE_INFO><year>2018</year><month>09</month><day>26</day><hour>06</hour><minute>43</minute><second>51</second><timezone>GMT+5.5</timezone><publisher>huxia</publisher></RELEASE_INFO>
    </VERSION>
    <FIRMWARE>
        <FW_ID>2354</FW_ID><FILESET_COUNT>1</FILESET_COUNT>
        <FILESET><FILE><FILENAME>update.zip</FILENAME><FILE_ID>1057</FILE_ID><SIZE>%size%</SIZE><CHECKSUM>%checksum%</CHECKSUM><FILE_VERSION>1</FILE_VERSION><INDEX>0</INDEX></FILE></FILESET>
    </FIRMWARE>
    <SPOP_LIST>
        <SPOP><SPOP_TYPE>2</SPOP_TYPE><SPOP_DATA>{"format version":1,"data":{"time":64800}}</SPOP_DATA></SPOP>
        <SPOP><SPOP_TYPE>6</SPOP_TYPE><SPOP_DATA>{"format version":1,"data":{"check":false,"download":true}}</SPOP_DATA></SPOP>
        <SPOP><SPOP_TYPE>7</SPOP_TYPE><SPOP_DATA>{"format version":1,"data":{"type":"mandatory","typed_data":{"download":"auto","install":"auto"}}}</SPOP_DATA></SPOP>
        <SPOP_NB>3</SPOP_NB>
    </SPOP_LIST>
    <DESCRIPTION>
        <en>English Description</en>
        <zh>中文描述</zh>
        <fr>la qualité</fr>
        <de>Qualität</de>
        <es>población está</es>
        <ru>Сегодня</ru>
        <ar>الحياة.</ar>
    </DESCRIPTION>
</GOTU>"#;

const DOWNLOAD_REQ_RESPONSE_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<GOTU>
    <FILE_LIST><FILE><FILE_ID>1057</FILE_ID><DOWNLOAD_URL>%filepath%</DOWNLOAD_URL></FILE></FILE_LIST>
    <SLAVE_LIST><SLAVE>%slave%</SLAVE></SLAVE_LIST>
</GOTU>"#;

pub fn get_full_check_response(
    curef: &str,
    target: &str,
    version: &str,
    size: &str,
    checksum: &str,
) -> String {
    CHECK_RESPONSE_TEMPLATE
        .to_string()
        .replace("%target%", target)
        .replace("%curef%", curef)
        .replace("%version%", version)
        .replace("%size%", size)
        .replace("%checksum%", checksum)
}

pub fn get_download_req_response(filepath: &str, slave: &str) -> String {
    DOWNLOAD_REQ_RESPONSE_TEMPLATE
        .to_string()
        .replace("%filepath%", filepath)
        .replace("%slave%", slave)
}
