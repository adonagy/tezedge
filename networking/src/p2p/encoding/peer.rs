use std::mem::size_of;

use log::debug;
use serde::{Deserialize, Serialize};

use tezos_encoding::encoding::{Encoding, Field, HasEncoding, Tag, TagMap};
use tezos_encoding::ser;

use crate::p2p::message::JsonMessage;

use super::current_branch::CurrentBranchMessage;
use super::current_head::CurrentHeadMessage;
use super::get_current_branch::GetCurrentBranchMessage;
use super::get_current_head::GetCurrentHeadMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum PeerMessage {
    Disconnect,
    //    Advertise,      // TODO
//    SwapRequest,    // TODO
//    SwapAck,        // TODO
    Bootstrap,

    GetCurrentBranch(GetCurrentBranchMessage),
    CurrentBranch(CurrentBranchMessage),
//    Deactivate,     // TODO
    GetCurrentHead(GetCurrentHeadMessage),
    CurrentHead(CurrentHeadMessage),
//    GetBlockHeaders,  // TODO
//    BlockHeader,      // TODO
//    GetOperations,    // TODO
//    Operation(OperationMessage),
//    GetProtocols,     // TODO
//    Protocol,         // TODO
//    GetOperationHashesForBlocks,    // TODO
//    OperationHashesForBlock,        // TODO
//    GetOperationsForBlocks,         // TODO
//    OperationsForBlock,             // TODO
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PeerMessageResponse {
    messages: Vec<PeerMessage>,
}

impl PeerMessageResponse {
    pub fn get_messages(&self) -> &Vec<PeerMessage> {
        &self.messages
    }
}

impl HasEncoding for PeerMessageResponse {
    fn encoding() -> Encoding {
        Encoding::Obj(vec![
            Field::new("messages", Encoding::dynamic(Encoding::list(
                Encoding::Tags(
                    size_of::<u16>(),
                    TagMap::new(&[
                        Tag::new(0x01, "Disconnect", Encoding::Unit),
                        Tag::new(0x02, "Bootstrap", Encoding::Unit),
                        Tag::new(0x10, "GetCurrentBranch", GetCurrentBranchMessage::encoding()),
                        Tag::new(0x11, "CurrentBranch", CurrentBranchMessage::encoding()),
                        Tag::new(0x13, "GetCurrentHead", GetCurrentHeadMessage::encoding()),
                        Tag::new(0x14, "CurrentHead", CurrentHeadMessage::encoding()),
                    ])
                )
            )))
        ])
    }
}

impl From<PeerMessage> for PeerMessageResponse {
    fn from(peer_message: PeerMessage) -> Self {
        PeerMessageResponse { messages: vec![peer_message] }
    }
}

pub fn log(peer_message: &PeerMessage) -> Result<(), ser::Error> {
    let json = match peer_message {
        PeerMessage::Bootstrap | PeerMessage::Disconnect => None,
        PeerMessage::GetCurrentBranch(msg) => Some(msg.as_json()?),
        PeerMessage::CurrentBranch(msg) => Some(msg.as_json()?),
        PeerMessage::GetCurrentHead(msg) => Some(msg.as_json()?),
        PeerMessage::CurrentHead(msg) => Some(msg.as_json()?),
    };

    if let Some(json) = json {
        debug!("Message received from peer: as JSON: \n{}", json);
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use failure::Error;

    use tezos_encoding::binary_writer::BinaryWriter;

    use crate::p2p::message::BinaryMessage;

    use super::*;

    #[test]
    fn can_deserialize_current_branch_message() {
        let message_bytes = hex::decode("0000127800118eceda2f000000ce000306f80146a6aefde9243ae18b191a8d010b7237d5130b3530ce5d1f60457411b2fa632d000000005c73d01c04acecbfac449678f1d68b90c7b7a86c9280fd373d872e072f3fb1b395681e71490000001100000001000000000800000000005ba1ca934484026d24be9ad40c98341c20e51092dd62bbf470bb9ff85061fa981ebbd90000000000031b4f9aff00c6d9a5d1fbf5eda49a01e52017dc78ca1d7a45f3f4fe32840052f9845a61ccdd6cf20139cedef0ed52395a327ad13390d9e8c1e999339a24f8513fe513ed689a46a6aefde9243ae18b191a8d010b7237d5130b3530ce5d1f60457411b2fa632d9aeb8e663111c3e5d3406bbf263a2d5869475ea8552bf16b28ef26a3ffac590a58f26ddf689bdc4547de09bc2ddb8e1e7a7a0646e40a49873578525c798c42e4c89f1799339c0dc8daa87f370d3a9a9ab4299a5d9d9082e1cfd3cd0cf1986f3f7543a65cd9bb6c0a96cd881cfcfd720178d859de8bceb4254bae78f29f0202773aeddd330be233bde3b84900cddff0546c952c3e32c36b1d27f96179c339230bf76cb1d94f23b8ba8542122e7a8a19d1e4f683f7961daed8eaf67897991a1a4de78712518593773de4b3c20ff3892c0bad466374ee96f452d76b1fa5ddd776f534505c1a16e7eea2cc8d75c484d67296678401b21cdc1c18ab4be2354ac2d83f85c2cc6844fe52989734d425f57dea06151085db0c37f39030c4cfbefc8d8a045d3a8c29b88d91c15a47e51b8e793845c00dcaf7b199f4030c43d561e10b3a24bec9b94c48f24a7641cdcce20ba3bd2fd2626d45e939098bd6ec36e4b000aae3babad329a056ebd8793270212913874adaba0141b67ddddd65128318303ef7bff158a78591c2c52ca7b9a0c4fbf06631565c3a8f823248fd91ebdc873d5f1d884e66aef6b7866a94c4eded8e8b4ecd5352b15f5c97a59fda96a4964422e7fe2c3077c471b8da1fe32ca4d6741f58bd848e332e0e51653109d345edbeda5460f9e816dccaf0836d4c1176dbbba0a6b91445b3ccecee204b542b1bcb05cd06977d845910c90a00ec90188228ae47d0a16a1ab95b2d91b21c876f5c5ea179bb3f3410808fd5cf4aef34d38a2442819daa51d3c33dd30418502e686245fffacd6cede9a686c6b79fb6e17c83b48829c12f073049434a574e21a3f1776b68bf65a0366f32bdc144e86eb40feac4a48804b6e0cd5548f5edad790336fd29354b737b129d7fdc7b6fa4049e4c570961d1e23926c5acae7b763cdbc805f3f27f7726dc347573ca9b083f8268b148037bca6bbba46237e1083e07e8aee4621f2802c9ef50ba576a33e3c8673d75d9c0df662f7884ecf8d668fcfe61ed05077de4e624406a81a3c7f1bb9ef4aa9589620b48d4d3489f2bf94b738024af3ac7ecff13b9067d47b4562ebb9e14579d0df81a74802856020c91d94ca50f21dc20b660d8d9121689b7e967a47a1712a10f334762211ba39cc84c0b93d909f4b762abdac509d9fc629e4fcbe252180fda831a535e10bef8f34dc999842c37c57dd995e16d09c1198c063de759b4179ea0a39fc4acf7fb9f7038606cc0dd1f69f4c7cf13fc7e2ecf9c41b49817bc6388a7f7bd02ac1f6d48e948133c4bcb1ac9291b7c3d3c6da243db457cefd3e9e858a6fb1ddd87bf17185cf2d8e80807767cb6d81923b700aa86747e00dea299d0d5fea6110468ddd369ce564175110d3f0c4c1992a945ecb5b6f80b43fab2b756dc62530e144e5037f879ecdad2b8fc1934577d1360bdffe8fc393e02ccd8af1b4e60302894ce6efb12266e04de3c67f0f820e847904a0a1d5648e4fad65f1ef8d9edb65bc7106580cd69b3253d1ee1a14deda1102b16ff5f191c69642d29873df9ff44a9b72f8ff8431fc2d09c6f5bc1bc06ad8a67e66b7cce84cfdb363ab2261d4a4029a4a619f0b41d6c60b9e5476ab42e007be918e46251984f6c6598ccbf8c168c6c826dbf39d6cc2135c5c1b121bc71ae49dccaf070d3e356348d0283e6922b1379dde5434cbfc470466593b36b1589fde46be2142bfa3ad77694af14d6d9ec37d6666a1abea506ad199155f1b76e7cf53c0634b44ae294581263482d52fe6d9190200c6437c7dc8256ccea74afb960f1d9525218fb5a6b22c6eb2e84a81483077b5ccebd78c5eedbcdc09a8b21fba0ec5033087632d66db9dc5a5028efa085c5006abe83cb01bc64782b7d35d7f464eb1f9c0ef8da0686367daf1185d0b7c44542b9648568bbc9fd1e25a7cacf1b11aa75dc5e030a495ba32973f9ae9a4dc7df3ac5816e5b5ed86ec64dbea3c11455dd725cede53d76aa1dae91af6add713e2f8592d82035e3ea2735427f199186b977bcf895455b1b3187d1835bea62586560697ff8200fa8f6d9cd4b19b72c0af8608b9f59279914b13316c08f8b86ea6ae45333e578beb9e935340d833b32e67a44cf11b0502ec30cb8de665d23277cb1d84e60ecd3220161e050b4af34cd04bece5d62b96d947ca2d46b93a88e1bb8829ced792de8615749f5a1ff47ecea00847cfac403c61276d5ece498b5b5e317c2f9b04c8a77855c198b74b8fe2230cdadf9d46d4667beb5de17e99835e90e49188077c7cf235b3a7da3d25da02b64d53170458bf1850fd9188e62bfb42b62020631cf26541cec29b450bb6512c0e0a02dfc9b51621c45709328ce4b730217926038c01202b8bad2b090a7b96f772ad65f8f96d8cf6a0d2cc86654defff2e19598bb4d12f91035915248bec4b3ab96fd698b588092b8c817fa6343397ac951ef3b21c0d9fbddd3de37a71a7c384d8c2aab928a0f5c4150c196213d1e9d2c503eb0fd509d80131676f72a0181286dd7920b28d140bab35802205be190d10887dc9db6a263c4ac8d9687e04c583efcc18c43e389c3996706468377cb4433cebecc70e3abe168311a0e4968130fa8e85931629741a914e0728d03730e48cd72f8eb1aa1141b3d5abfdca5ee6aaded702e2475e3f3e7277038d4a515b18c35b20adafb9765e6414f95c38d2cbf6e8a1b5710aeb66e0f99ebcf6ba9bf0f95c023444c98ee48b1a289d6c3352e355dd06fc1cdb898ed37edf78e01f58ad0fd14a535d325c307be4f1177ce72ff1d70cd6fbcf635727a968c78a1ccad0d762c7d15364b152290b0cdea403283ef60520477172f6db1d2180bdd32ca0a194085d61bbc3cb50a6a3c905fab7daaaccc92ca0a3525c23592e0861337df759fe8eb93df114b28b94d5a4c26e9635fbe9f9b91985c522dcc886b9c582589fa1781437b6991b63821f3aaa2a2f3d94df40e21c20b42cd30393f639b065273cd33fe56419165f63a89b23c6189c426fe4c451e1d6afff82bdb8842b42314d99372a7cc3962f0efe77921301c92f4084bb8207c1c96d241416883e276ad4bc8d6b1051e6a11f0827458368fef27cda6760933b321c372b00255c31333997a96cb78c7fbd82a1b905c7e87ae4aa7066c13a4c21cbf09a0a5c345433373b81ab818bc6f6d22964883d4adc3f16d61cd1514baf9a8301add991c83c0cf10c7ba641a11ccc2789680d37cac29ebb9c07ad31567f733f2df978710d5fd768b60276ec2d5129e72813f0cb9efc569aa73e19b57aa623063bab01ddd98c53a13c85c7909eb626ff3ffa37ce8a7b10f235f99f0ec7b533b7f537be7ded4c08b30976bbac292a4e8f4bb85a83edb53ceb978c7f615cbc1101df39b74697dadd90fa7cc8fc5102fc483026bbf3c66f0749a90c16fe3622558bc6999ebde5ba64ca890f71430b402b8c9ba012ad793a1c70b141b48a07fbbb525965d949c992725740c07d1415ef9b26fe63d50a67c6d4979d68f8dad3e32cb03e26e4e4e64d50f94ab17286173825374a9f6e96ff50466c2b699a38fc69e5a79e7e319a62693ec85e2a8b2ef77a24de96bfab7d7a76d343a569c8b2e572d77757589b9d2a11e8ea8ff56e22ebae7c883053c8db992684ba05f0a6574f8162e480a32b8882d489a7a8d4313caadbcf44418d400983acf65f952ece41d21fee18c67b12a26949294111bbed41d88aa5e26a78bdbdb509e664f431f817c33f8a22b0d9a2110d16159fcdcaf000a70e51c6cdc3008549a48c47091aa2f8320ba4b8e060b71591da10abc7f5e080c92c2d7537a29804755fd50c02cbad30687b4cb66b2d0eaa9b82dc75daf8f685ad3f8cdcdae9c02d60f4218f008777a4bf505015bfdeb7647f1869b45095c298ae4f16cf11518a778716d6f7972e954aeb3c6774550e41534f1c8fae506bba6cd233efd13c8ab72be51b345f6132fbf0e38d88457254d877da235e168d8f1d97e5edac77fad58ae4189da88534ec437b619cab43302519c7d654edd6d42a0bbfb891593fb9ad3526bb8dc7a38c8ecc3fe591bfa3e0750ec23751475c88678fb1109483e1f7661695a727ce0397a1ef0e7856a6ed253df9e97a7cd1c5dc14534fb296f0cd58b93fa142d771d1db1df1c3a9188bd1a3a27ba08ffe1b340fa70dfb4fbc3bf47acbda083c110f07b3c479717d738271a30ec44e550572024b0fa23a48165542ac931606e9716fa6a8a7d5b70982b533b649f3624b0221a96c69263e5bd844b04724e1b68242b01ef8daa8bd5bf02e293779af56807c40184c1192fe1c9c1ebf0da4906f3c319f84afe57890bacf65947fada4b70a03323e955e529ae9127a2b2bff2d6f7afd14301035b2656ccf6d0e44683bac4760c370c7339513ea55ef7a0b24e939338215a82dfb7fbc8af11b8b207148955330628f19a77a4b7061106dbfc6c0598ab111c598126dc61c1fc1f34e8fe046731f05ad52a614cd91e8672c9dc6889a37d6b198d757272dcb8c4c9b024d3ac6962eded524f9e281780a3e149cb406bce50f6de5988f9bdf29f3c1c2f1deb4b13407b63a3900148d48e26ed33093a1f99394f1fafea588b79c7d516ea9e0d5955f44e07dca183a6dbc5f4c562697b0b3ec37d8d63493624774b283b9aabf22aac52e5200acd3c89fddbd16a23cdc1e1f081d08c3c9277d43b3bf2ce488a563350e07b1d89cf3753ff777e272344684200d4a3d5b3afc6f8ddd2be6f9c0ad32c3922733d6461cf0446c7bfd99f2f32a3189cd8882ffe6aabf39d08b43eb37dd6de92a92e9484ec8b6b6c823502fa6b6780a3924b5ea0e93bc07a7261e78d72440f2a16a614d3f29cf0951b561e76e6bacc51030257370b813ef356d76fa61e99b82af73b36365017f4b03cd537ea6aceca48c3cc2b34f163beb21203f0498fd7ec42e309463ea343cdddd326234517d3705b05dc3e8d66132d039fe6a7461e09196b6859606ed18391809b1d5691793beb77ded910997e667a1eec275446ddae8cd6015c632acf5d9f92f014b5da5497e23a07c0d4413e426a57005d8a79eb2dc11b99410c1858db28c55769d7724027665984c98b6dacc79a8aafde11d50bd0b87253c267821651c302e3d993bfb0e52656b5278b96c00c3474d7e2632a9936551371578840a5a999999863fda5ef6e8d04b0ddd807d4905c16c3449580622fc0fa5288f8039cd0cf7ca0f591acc6eb4fbada88c7fdd273b736b27b5305ee25c079cc18a5c1956793302cf8d679b26d22593f9f7858c5ff95f03a8e738652a892b89ec667e87bb35dbc552e3a6123325c94308dd4580fc91111a64698b8a18e36b48f8d0c770c2c1374a4fee29693cec76a3dc724894691916cb10d06dba3207d6c67d1ae49233a25a685bd23b549e1d756904e925a42db2b00fb56c8e4f94ff9b4af7d65b8d9fc46108878c823aa94d76b8b55a4c8d0a8379d74b2eff1a4252a57150f2233037af553c9404f3bef48e7e4b34db072ec28c5d160bb1b7967d00ea088117b6d34fb3a67e41e16f6f9c09b45760786168cc741e43bb4b73f095257503ca15ffc84097754f42633388e8959b01e2135edcf43c455f5ebb395b1c2dcd9c99ed8856415681c16f43b71caeae745a0e60933bdaa98d0ca720fd52861eefde238d5f63e49a2ef8b936472ac00c430edb8e4298da4df3bc18fb156d9495127db36c6240883c6858c25eaa2178443aca5d1b4c3dfea078773d8833fbb6b649df8136245b6372fab1e45ce78031349df0e3a4f259768d4a948aea689485f8717cf126a836cbabcb14cdd850645e37aad3fe735588e4311dfbc2587ff9ef1c4c23c6b0f3f0c44570e9654e2d77eaf2e87558ef06d9570930d5ade7198a4f4725b354266aa699aaf18fb241c5daa2fce132ff4b5217aa8c977bfcb7e8ded6207a88919559e681b1e9ffc745958f504074740ddedb7c3bc162290ee73fa0563f03648c8975ed43a2f97b2c001bea83484fc7396192de64b90e855ce3f0c193c93416c7eb0b5821f16d99a046687e18a6f6ba0e35725412714d15b354ab8f3de8a1c462b82070568d617e203415b414050feea9442f310d461814930cd28dd9d3eda8cdf4258c40df5ec8f3d8eb9a033b3a8d00b18b9ed04552eedf5efea93f6adbf2e6c117a6904478b0dab56d49ee382507aba19bf48ee1685f29d2e9f0636dd24d88a28dba43bc035720d1ba70b2186b160d386bb08037dfa7130f19d369a9d94ebfa8796d5f64f15bf3d894e7a882f14124a40b5e2898f454e4fbd2a3fd3ece11641dad2d0da8fcf233671b6a04fcf679d2a381c2544ea6c1ea29ba6157776ed8424affa610d").unwrap();
        let messages = PeerMessageResponse::from_bytes(message_bytes).unwrap();
        assert_eq!(1, messages.get_messages().len());
        let message = messages.get_messages().get(0).unwrap();

        match message {
            PeerMessage::CurrentBranch(current_branch_message) => {
                assert_eq!(&hex::decode("8eceda2f").unwrap(), current_branch_message.get_chain_id());

                let current_head = current_branch_message.get_current_branch().get_current_head();
                assert_eq!(198392, current_head.get_level());
                assert_eq!(4, current_head.get_validation_pass());

                assert_eq!(&hex::decode("46a6aefde9243ae18b191a8d010b7237d5130b3530ce5d1f60457411b2fa632d").unwrap(), current_head.get_predecessor());
                assert_eq!(&hex::decode("934484026d24be9ad40c98341c20e51092dd62bbf470bb9ff85061fa981ebbd9").unwrap(), current_head.get_context());
                assert_eq!(&hex::decode("acecbfac449678f1d68b90c7b7a86c9280fd373d872e072f3fb1b395681e7149").unwrap(), current_head.get_operations_hash());


                assert_eq!(2, current_head.get_fitness().len());
                let fitness_0 = current_head.get_fitness().get(0).unwrap();
                let expected_fitness_0 = hex::decode("00").unwrap();
                assert_eq!(&expected_fitness_0, fitness_0);
                let fitness_1 = current_head.get_fitness().get(1).unwrap();
                let expected_fitness_1 = hex::decode("00000000005ba1ca").unwrap();
                assert_eq!(&expected_fitness_1, fitness_1);

                assert_eq!(75, current_head.get_protocol_data().len());
                let expected_protocol_data = hex::decode("0000000000031b4f9aff00c6d9a5d1fbf5eda49a01e52017dc78ca1d7a45f3f4fe32840052f9845a61ccdd6cf20139cedef0ed52395a327ad13390d9e8c1e999339a24f8513fe513ed689a").unwrap();
                assert_eq!(&expected_protocol_data, current_head.get_protocol_data());

                assert_eq!(141, current_branch_message.get_current_branch().get_history().len());

                let mut writer = BinaryWriter::new();
                let bytes = writer.write(current_branch_message, &CurrentBranchMessage::encoding()).unwrap();

                // without length: 00001278 and without tag 0011
                assert_eq!(
                    hex::decode("8eceda2f000000ce000306f80146a6aefde9243ae18b191a8d010b7237d5130b3530ce5d1f60457411b2fa632d000000005c73d01c04acecbfac449678f1d68b90c7b7a86c9280fd373d872e072f3fb1b395681e71490000001100000001000000000800000000005ba1ca934484026d24be9ad40c98341c20e51092dd62bbf470bb9ff85061fa981ebbd90000000000031b4f9aff00c6d9a5d1fbf5eda49a01e52017dc78ca1d7a45f3f4fe32840052f9845a61ccdd6cf20139cedef0ed52395a327ad13390d9e8c1e999339a24f8513fe513ed689a46a6aefde9243ae18b191a8d010b7237d5130b3530ce5d1f60457411b2fa632d9aeb8e663111c3e5d3406bbf263a2d5869475ea8552bf16b28ef26a3ffac590a58f26ddf689bdc4547de09bc2ddb8e1e7a7a0646e40a49873578525c798c42e4c89f1799339c0dc8daa87f370d3a9a9ab4299a5d9d9082e1cfd3cd0cf1986f3f7543a65cd9bb6c0a96cd881cfcfd720178d859de8bceb4254bae78f29f0202773aeddd330be233bde3b84900cddff0546c952c3e32c36b1d27f96179c339230bf76cb1d94f23b8ba8542122e7a8a19d1e4f683f7961daed8eaf67897991a1a4de78712518593773de4b3c20ff3892c0bad466374ee96f452d76b1fa5ddd776f534505c1a16e7eea2cc8d75c484d67296678401b21cdc1c18ab4be2354ac2d83f85c2cc6844fe52989734d425f57dea06151085db0c37f39030c4cfbefc8d8a045d3a8c29b88d91c15a47e51b8e793845c00dcaf7b199f4030c43d561e10b3a24bec9b94c48f24a7641cdcce20ba3bd2fd2626d45e939098bd6ec36e4b000aae3babad329a056ebd8793270212913874adaba0141b67ddddd65128318303ef7bff158a78591c2c52ca7b9a0c4fbf06631565c3a8f823248fd91ebdc873d5f1d884e66aef6b7866a94c4eded8e8b4ecd5352b15f5c97a59fda96a4964422e7fe2c3077c471b8da1fe32ca4d6741f58bd848e332e0e51653109d345edbeda5460f9e816dccaf0836d4c1176dbbba0a6b91445b3ccecee204b542b1bcb05cd06977d845910c90a00ec90188228ae47d0a16a1ab95b2d91b21c876f5c5ea179bb3f3410808fd5cf4aef34d38a2442819daa51d3c33dd30418502e686245fffacd6cede9a686c6b79fb6e17c83b48829c12f073049434a574e21a3f1776b68bf65a0366f32bdc144e86eb40feac4a48804b6e0cd5548f5edad790336fd29354b737b129d7fdc7b6fa4049e4c570961d1e23926c5acae7b763cdbc805f3f27f7726dc347573ca9b083f8268b148037bca6bbba46237e1083e07e8aee4621f2802c9ef50ba576a33e3c8673d75d9c0df662f7884ecf8d668fcfe61ed05077de4e624406a81a3c7f1bb9ef4aa9589620b48d4d3489f2bf94b738024af3ac7ecff13b9067d47b4562ebb9e14579d0df81a74802856020c91d94ca50f21dc20b660d8d9121689b7e967a47a1712a10f334762211ba39cc84c0b93d909f4b762abdac509d9fc629e4fcbe252180fda831a535e10bef8f34dc999842c37c57dd995e16d09c1198c063de759b4179ea0a39fc4acf7fb9f7038606cc0dd1f69f4c7cf13fc7e2ecf9c41b49817bc6388a7f7bd02ac1f6d48e948133c4bcb1ac9291b7c3d3c6da243db457cefd3e9e858a6fb1ddd87bf17185cf2d8e80807767cb6d81923b700aa86747e00dea299d0d5fea6110468ddd369ce564175110d3f0c4c1992a945ecb5b6f80b43fab2b756dc62530e144e5037f879ecdad2b8fc1934577d1360bdffe8fc393e02ccd8af1b4e60302894ce6efb12266e04de3c67f0f820e847904a0a1d5648e4fad65f1ef8d9edb65bc7106580cd69b3253d1ee1a14deda1102b16ff5f191c69642d29873df9ff44a9b72f8ff8431fc2d09c6f5bc1bc06ad8a67e66b7cce84cfdb363ab2261d4a4029a4a619f0b41d6c60b9e5476ab42e007be918e46251984f6c6598ccbf8c168c6c826dbf39d6cc2135c5c1b121bc71ae49dccaf070d3e356348d0283e6922b1379dde5434cbfc470466593b36b1589fde46be2142bfa3ad77694af14d6d9ec37d6666a1abea506ad199155f1b76e7cf53c0634b44ae294581263482d52fe6d9190200c6437c7dc8256ccea74afb960f1d9525218fb5a6b22c6eb2e84a81483077b5ccebd78c5eedbcdc09a8b21fba0ec5033087632d66db9dc5a5028efa085c5006abe83cb01bc64782b7d35d7f464eb1f9c0ef8da0686367daf1185d0b7c44542b9648568bbc9fd1e25a7cacf1b11aa75dc5e030a495ba32973f9ae9a4dc7df3ac5816e5b5ed86ec64dbea3c11455dd725cede53d76aa1dae91af6add713e2f8592d82035e3ea2735427f199186b977bcf895455b1b3187d1835bea62586560697ff8200fa8f6d9cd4b19b72c0af8608b9f59279914b13316c08f8b86ea6ae45333e578beb9e935340d833b32e67a44cf11b0502ec30cb8de665d23277cb1d84e60ecd3220161e050b4af34cd04bece5d62b96d947ca2d46b93a88e1bb8829ced792de8615749f5a1ff47ecea00847cfac403c61276d5ece498b5b5e317c2f9b04c8a77855c198b74b8fe2230cdadf9d46d4667beb5de17e99835e90e49188077c7cf235b3a7da3d25da02b64d53170458bf1850fd9188e62bfb42b62020631cf26541cec29b450bb6512c0e0a02dfc9b51621c45709328ce4b730217926038c01202b8bad2b090a7b96f772ad65f8f96d8cf6a0d2cc86654defff2e19598bb4d12f91035915248bec4b3ab96fd698b588092b8c817fa6343397ac951ef3b21c0d9fbddd3de37a71a7c384d8c2aab928a0f5c4150c196213d1e9d2c503eb0fd509d80131676f72a0181286dd7920b28d140bab35802205be190d10887dc9db6a263c4ac8d9687e04c583efcc18c43e389c3996706468377cb4433cebecc70e3abe168311a0e4968130fa8e85931629741a914e0728d03730e48cd72f8eb1aa1141b3d5abfdca5ee6aaded702e2475e3f3e7277038d4a515b18c35b20adafb9765e6414f95c38d2cbf6e8a1b5710aeb66e0f99ebcf6ba9bf0f95c023444c98ee48b1a289d6c3352e355dd06fc1cdb898ed37edf78e01f58ad0fd14a535d325c307be4f1177ce72ff1d70cd6fbcf635727a968c78a1ccad0d762c7d15364b152290b0cdea403283ef60520477172f6db1d2180bdd32ca0a194085d61bbc3cb50a6a3c905fab7daaaccc92ca0a3525c23592e0861337df759fe8eb93df114b28b94d5a4c26e9635fbe9f9b91985c522dcc886b9c582589fa1781437b6991b63821f3aaa2a2f3d94df40e21c20b42cd30393f639b065273cd33fe56419165f63a89b23c6189c426fe4c451e1d6afff82bdb8842b42314d99372a7cc3962f0efe77921301c92f4084bb8207c1c96d241416883e276ad4bc8d6b1051e6a11f0827458368fef27cda6760933b321c372b00255c31333997a96cb78c7fbd82a1b905c7e87ae4aa7066c13a4c21cbf09a0a5c345433373b81ab818bc6f6d22964883d4adc3f16d61cd1514baf9a8301add991c83c0cf10c7ba641a11ccc2789680d37cac29ebb9c07ad31567f733f2df978710d5fd768b60276ec2d5129e72813f0cb9efc569aa73e19b57aa623063bab01ddd98c53a13c85c7909eb626ff3ffa37ce8a7b10f235f99f0ec7b533b7f537be7ded4c08b30976bbac292a4e8f4bb85a83edb53ceb978c7f615cbc1101df39b74697dadd90fa7cc8fc5102fc483026bbf3c66f0749a90c16fe3622558bc6999ebde5ba64ca890f71430b402b8c9ba012ad793a1c70b141b48a07fbbb525965d949c992725740c07d1415ef9b26fe63d50a67c6d4979d68f8dad3e32cb03e26e4e4e64d50f94ab17286173825374a9f6e96ff50466c2b699a38fc69e5a79e7e319a62693ec85e2a8b2ef77a24de96bfab7d7a76d343a569c8b2e572d77757589b9d2a11e8ea8ff56e22ebae7c883053c8db992684ba05f0a6574f8162e480a32b8882d489a7a8d4313caadbcf44418d400983acf65f952ece41d21fee18c67b12a26949294111bbed41d88aa5e26a78bdbdb509e664f431f817c33f8a22b0d9a2110d16159fcdcaf000a70e51c6cdc3008549a48c47091aa2f8320ba4b8e060b71591da10abc7f5e080c92c2d7537a29804755fd50c02cbad30687b4cb66b2d0eaa9b82dc75daf8f685ad3f8cdcdae9c02d60f4218f008777a4bf505015bfdeb7647f1869b45095c298ae4f16cf11518a778716d6f7972e954aeb3c6774550e41534f1c8fae506bba6cd233efd13c8ab72be51b345f6132fbf0e38d88457254d877da235e168d8f1d97e5edac77fad58ae4189da88534ec437b619cab43302519c7d654edd6d42a0bbfb891593fb9ad3526bb8dc7a38c8ecc3fe591bfa3e0750ec23751475c88678fb1109483e1f7661695a727ce0397a1ef0e7856a6ed253df9e97a7cd1c5dc14534fb296f0cd58b93fa142d771d1db1df1c3a9188bd1a3a27ba08ffe1b340fa70dfb4fbc3bf47acbda083c110f07b3c479717d738271a30ec44e550572024b0fa23a48165542ac931606e9716fa6a8a7d5b70982b533b649f3624b0221a96c69263e5bd844b04724e1b68242b01ef8daa8bd5bf02e293779af56807c40184c1192fe1c9c1ebf0da4906f3c319f84afe57890bacf65947fada4b70a03323e955e529ae9127a2b2bff2d6f7afd14301035b2656ccf6d0e44683bac4760c370c7339513ea55ef7a0b24e939338215a82dfb7fbc8af11b8b207148955330628f19a77a4b7061106dbfc6c0598ab111c598126dc61c1fc1f34e8fe046731f05ad52a614cd91e8672c9dc6889a37d6b198d757272dcb8c4c9b024d3ac6962eded524f9e281780a3e149cb406bce50f6de5988f9bdf29f3c1c2f1deb4b13407b63a3900148d48e26ed33093a1f99394f1fafea588b79c7d516ea9e0d5955f44e07dca183a6dbc5f4c562697b0b3ec37d8d63493624774b283b9aabf22aac52e5200acd3c89fddbd16a23cdc1e1f081d08c3c9277d43b3bf2ce488a563350e07b1d89cf3753ff777e272344684200d4a3d5b3afc6f8ddd2be6f9c0ad32c3922733d6461cf0446c7bfd99f2f32a3189cd8882ffe6aabf39d08b43eb37dd6de92a92e9484ec8b6b6c823502fa6b6780a3924b5ea0e93bc07a7261e78d72440f2a16a614d3f29cf0951b561e76e6bacc51030257370b813ef356d76fa61e99b82af73b36365017f4b03cd537ea6aceca48c3cc2b34f163beb21203f0498fd7ec42e309463ea343cdddd326234517d3705b05dc3e8d66132d039fe6a7461e09196b6859606ed18391809b1d5691793beb77ded910997e667a1eec275446ddae8cd6015c632acf5d9f92f014b5da5497e23a07c0d4413e426a57005d8a79eb2dc11b99410c1858db28c55769d7724027665984c98b6dacc79a8aafde11d50bd0b87253c267821651c302e3d993bfb0e52656b5278b96c00c3474d7e2632a9936551371578840a5a999999863fda5ef6e8d04b0ddd807d4905c16c3449580622fc0fa5288f8039cd0cf7ca0f591acc6eb4fbada88c7fdd273b736b27b5305ee25c079cc18a5c1956793302cf8d679b26d22593f9f7858c5ff95f03a8e738652a892b89ec667e87bb35dbc552e3a6123325c94308dd4580fc91111a64698b8a18e36b48f8d0c770c2c1374a4fee29693cec76a3dc724894691916cb10d06dba3207d6c67d1ae49233a25a685bd23b549e1d756904e925a42db2b00fb56c8e4f94ff9b4af7d65b8d9fc46108878c823aa94d76b8b55a4c8d0a8379d74b2eff1a4252a57150f2233037af553c9404f3bef48e7e4b34db072ec28c5d160bb1b7967d00ea088117b6d34fb3a67e41e16f6f9c09b45760786168cc741e43bb4b73f095257503ca15ffc84097754f42633388e8959b01e2135edcf43c455f5ebb395b1c2dcd9c99ed8856415681c16f43b71caeae745a0e60933bdaa98d0ca720fd52861eefde238d5f63e49a2ef8b936472ac00c430edb8e4298da4df3bc18fb156d9495127db36c6240883c6858c25eaa2178443aca5d1b4c3dfea078773d8833fbb6b649df8136245b6372fab1e45ce78031349df0e3a4f259768d4a948aea689485f8717cf126a836cbabcb14cdd850645e37aad3fe735588e4311dfbc2587ff9ef1c4c23c6b0f3f0c44570e9654e2d77eaf2e87558ef06d9570930d5ade7198a4f4725b354266aa699aaf18fb241c5daa2fce132ff4b5217aa8c977bfcb7e8ded6207a88919559e681b1e9ffc745958f504074740ddedb7c3bc162290ee73fa0563f03648c8975ed43a2f97b2c001bea83484fc7396192de64b90e855ce3f0c193c93416c7eb0b5821f16d99a046687e18a6f6ba0e35725412714d15b354ab8f3de8a1c462b82070568d617e203415b414050feea9442f310d461814930cd28dd9d3eda8cdf4258c40df5ec8f3d8eb9a033b3a8d00b18b9ed04552eedf5efea93f6adbf2e6c117a6904478b0dab56d49ee382507aba19bf48ee1685f29d2e9f0636dd24d88a28dba43bc035720d1ba70b2186b160d386bb08037dfa7130f19d369a9d94ebfa8796d5f64f15bf3d894e7a882f14124a40b5e2898f454e4fbd2a3fd3ece11641dad2d0da8fcf233671b6a04fcf679d2a381c2544ea6c1ea29ba6157776ed8424affa610d").unwrap(),
                    bytes
                );
            }
            _ => panic!("Unsupported encoding: {:?}", message)
        }
    }

    #[test]
    fn can_deserialize_get_current_head_message_known_valid() -> Result<(), Error> {
        let message_bytes = hex::decode("0000010400148eceda2f000000ce0003be930116caa5bebae6c1997498bd90b2d2d6dcb14e2cc3a83b38067c784a0b485a4763000000005c8f572e049518937f78bbc2e2d460e7d26daa73c93763362c64c2059f5b7ecaba6e6f580d000000110000000100000000080000000000714aa08e289a17ee0bbd90ef57b80c52318829029fc9e17e4a782248755cdeaafd0dac000000000003e35a661200a75ebed94c886ce8c2700cc2fb38e301e7573f481eff49aea6892068cef7c9290947567e9df3a2cfc99ed9b0666f9c0291f586f65eb9e42cf4cdbef1ef8424d000000020c533d1d8a515b35fac67eb9926a6c983397208511ce69808d57177415654bf090000000400000000")?;
        let messages = PeerMessageResponse::from_bytes(message_bytes)?;
        assert_eq!(1, messages.get_messages().len());
        let message = messages.get_messages().get(0).unwrap();

        match message {
            PeerMessage::CurrentHead(current_head_message) => {
                assert_eq!(&hex::decode("8eceda2f")?, current_head_message.get_chain_id());

                let block_header = current_head_message.get_current_block_header();
                assert_eq!(245395, block_header.get_level());
                assert_eq!(1, block_header.get_proto());

                let expected_protocol_data = hex::decode("000000000003e35a661200a75ebed94c886ce8c2700cc2fb38e301e7573f481eff49aea6892068cef7c9290947567e9df3a2cfc99ed9b0666f9c0291f586f65eb9e42cf4cdbef1ef8424d0")?;
                assert_eq!(&expected_protocol_data, block_header.get_protocol_data());

                let mempool = current_head_message.get_current_mempool();
                assert_eq!(1, mempool.get_known_valid().len());
                let expected_known_valid = hex::decode("c533d1d8a515b35fac67eb9926a6c983397208511ce69808d57177415654bf09")?;
                assert_eq!(&expected_known_valid, mempool.get_known_valid().get(0).unwrap());
                assert_eq!(0, mempool.get_pending().len());
            }
            _ => panic!("Unsupported encoding: {:?}", message)
        }

        Ok(())
    }

    #[test]
    fn can_deserialize_get_current_head_message_pending() -> Result<(), Error> {
        let message_bytes = hex::decode("0000012400148eceda2f000000ce0003cad5019b6feff784b018e609632b3bb66248a13c9efcaaaef34ddb805f07b4b2191760000000005c917b62045a8e2646ea6a2cf1ed392de0d4e2a45696a9662c0af212de73b72544357d757f00000011000000010000000008000000000072a3ce162dfbadfbdc34b00d694ba656c165b660643f6af5216a2462c52f0c41d98ee8000000000003d671d0520032846bbd8b2d6e10daa9cb6d6f82e4070d9c8047b081ea80cd6a473a3868135229c7650b7a6401d7c83c1b8896662faeb361a7bbb2f881ede725630e1ce344830000000000000044000000403cebec53e6ff9207dd669c3777cec4e74feadcd5f0131c819d261cdb0d9b5d9470669010ec4053d96d750daefbcdc1f51ed79f9e29fb16931515eccb84cb6a55")?;
        let messages = PeerMessageResponse::from_bytes(message_bytes)?;
        assert_eq!(1, messages.get_messages().len());
        let message = messages.get_messages().get(0).unwrap();

        match message {
            PeerMessage::CurrentHead(current_head_message) => {
                assert_eq!(&hex::decode("8eceda2f")?, current_head_message.get_chain_id());

                let block_header = current_head_message.get_current_block_header();
                assert_eq!(248533, block_header.get_level());
                assert_eq!(1, block_header.get_proto());

                let expected_protocol_data = hex::decode("000000000003d671d0520032846bbd8b2d6e10daa9cb6d6f82e4070d9c8047b081ea80cd6a473a3868135229c7650b7a6401d7c83c1b8896662faeb361a7bbb2f881ede725630e1ce34483")?;
                assert_eq!(&expected_protocol_data, block_header.get_protocol_data());

                let mempool = current_head_message.get_current_mempool();
                assert_eq!(0, mempool.get_known_valid().len());
                assert_eq!(2, mempool.get_pending().len());
                let expected_pending = hex::decode("3cebec53e6ff9207dd669c3777cec4e74feadcd5f0131c819d261cdb0d9b5d94")?;
                assert_eq!(&expected_pending, mempool.get_pending().get(0).unwrap());
                let expected_pending = hex::decode("70669010ec4053d96d750daefbcdc1f51ed79f9e29fb16931515eccb84cb6a55")?;
                assert_eq!(&expected_pending, mempool.get_pending().get(1).unwrap());
            }
            _ => panic!("Unsupported encoding: {:?}", message)
        }

        Ok(())
    }

    #[test]
    fn can_deserialize_bootstrap() -> Result<(), Error> {
        let message_bytes = hex::decode("000000020002")?;
        let messages = PeerMessageResponse::from_bytes(message_bytes).unwrap();
        assert_eq!(1, messages.get_messages().len());
        let message = messages.get_messages().get(0).unwrap();

        match message {
            PeerMessage::Bootstrap => {
                Ok(())
            }
            _ => panic!("Unsupported encoding: {:?}", message)
        }
    }

    #[test]
    fn can_serialize_bootstrap() -> Result<(), Error> {
        let message = PeerMessageResponse::from(PeerMessage::Bootstrap);
        let serialized = hex::encode(message.as_bytes()?);
        let expected = "000000020002";
        Ok(assert_eq!(expected, &serialized))
    }
}