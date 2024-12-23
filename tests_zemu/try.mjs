import TransportNodeHid from '@ledgerhq/hw-transport-node-hid'
import { PenumbraApp } from '@zondax/ledger-penumbra'


async function main() {
  const transport = await TransportNodeHid.default.open()

  const app = new PenumbraApp(transport)

  const PEN_PATH = "m/44'/6532'/0'"
  const ACCOUNT_ID = 1
  const RANDOMIZER = '770187941264c925f8ba8776'
  const PLAN = '0abe020abb020aa8010a300a0a08f4979edff8bbd1990712220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10122014b6628b360cca59dad6c87bc2b082b694ce0db47bcda0cfaf6d49ca187ed80e1a520a50890bc98e3698aa4578e419b028da5672e627c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42d5f20b52f10f4f9b09cfad4251a20acbb274bf255bbd32da0f5c380299b3bfa97a533cc2ca46e99e913a6b7c28a0422204cc69a01753e22bfa0b4fb75902843396ab34e30d542abe32a022c00d65708042a2082b07699836e60da166a1d61247e22182059a0aefa9c6237796a0080ad6c05123220c87020807fef96d1be9611d5b667a4440149c88570402e69953ed3cc1b5b2b010abe020abb020aa8010a300a0a08fc859aaaa08b91a20212220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a1012206dfb50a0cf56ab7031b3033585625b5258bab62322c95628210c221d190c2db91a520a50890bc98e3698aa4578e419b028da5672e627c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42d5f20b52f1099baa080b4f9241a204749c0e0361387ba82a4c19bc67e0b5c06c8c1774cec1c40704410f555a07c01222007199b871976c500d0fd9c7d84080928b08d1cada04c8d1dcc0312ff9c8c09022a20347b2de85bb5b0e239698c25d3d48c518948329fb68b5191ae001557094063033220b80b022226dfe8cc2147b9c84611b3e176be6ed96923be009e3b55cb926a9c10121e0885d61c120a70656e756d6272612d311a0c0a0a08b0f1a2f1f4a7a6c3042a9a030af5020a520a50e3e26ee0d15a394b72006b8225947579987352a906d2c40e3dff8cc09d6073f1c16c201752ce1af319be3e8cd42f87e1566e0f02a7dea4dae9ca561a792b9c669f5d2f73baf62ecc5d3e90951d333292129e0252387a663820753635756c384320564849797561207361376c615335393630206f695931207920303020643220306e6253203548206c5831417270326d20206f7220304951642071384e724b204557564d59483574746b376d20202058365a6e7762344a3332204e674f58387a4f6920774c33684d4e7a6820416e6e204537206e6e576d437120332020204b343834653455624a3254394655516459324d662053652053202043206d30787a204641324d205875206c68454d5a68314348456b206c33376d4c4b6f56586820326b35577a504932204537207241647120554b3620393520335820473320386536524c734a20206b5731322033204d3656334a537034205036205573635456624539573230207931347971207a6633764378122023f300f488b9ece996037c9e2b1ede7fe3cf1260bd3a02820a36c4e6bda74af2'

  
  let resp = await app.deviceInfo()
  console.log('Device Info', resp)
  resp = await app.getVersion()
  console.log('Version', resp)

  resp = await app.showAddress(PEN_PATH, ACCOUNT_ID, RANDOMIZER)
  console.log(resp)
  const address = resp.address
  console.log(address.toString('hex'))

  const messageToSign = Buffer.from(PLAN, 'hex')
  try {
    const signatureRequest = app.sign(PEN_PATH, messageToSign)
    const signatureResponse = await signatureRequest
    console.log("Effect hash:", signatureResponse.effectHash.toString('hex'))

    if (signatureResponse.spendAuthSignatures.length > 0) {
      signatureResponse.spendAuthSignatures.forEach((signature, index) => {
        console.log(`Spend Auth Signature ${index + 1}: ${signature.toString('hex')}`);
      });
    } else {
      console.log("No spend auth signatures available.");
    }

    if (signatureResponse.delegatorVoteSignatures.length > 0) {
      signatureResponse.delegatorVoteSignatures.forEach((signature, index) => {
        console.log(`Delegator Vote Signature ${index + 1}: ${signature.toString('hex')}`);
      });
    } else {
      console.log("No delegator vote signatures available.");
    }

  } catch (e) {
    console.log(e)
  }


}

;(async () => {
  await main()
})()
