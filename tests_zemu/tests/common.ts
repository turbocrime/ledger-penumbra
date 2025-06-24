import { IDeviceModel, DEFAULT_START_OPTIONS } from '@zondax/zemu'

import { resolve } from 'path'

export const APP_SEED = 'equip will roof matter pink blind book anxiety banner elbow sun young'
export const PENUMBRA_PATH = "m/44'/6532'/0'"
export const ACCOUNT_ID = 1
export const ACCOUNT_ID2 = 2

const APP_PATH_X = resolve('../app/output/app_x.elf')
const APP_PATH_SP = resolve('../app/output/app_s2.elf')
const APP_PATH_ST = resolve('../app/output/app_stax.elf')
const APP_PATH_FL = resolve('../app/output/app_flex.elf')

export const models: IDeviceModel[] = [
  { name: 'nanox', prefix: 'X', path: APP_PATH_X },
  { name: 'nanosp', prefix: 'SP', path: APP_PATH_SP },
  { name: 'stax', prefix: 'ST', path: APP_PATH_ST },
  { name: 'flex', prefix: 'FL', path: APP_PATH_FL },
]

export const defaultOptions = {
  ...DEFAULT_START_OPTIONS,
  logging: true,
  custom: `-s "${APP_SEED}"`,
  X11: false,
}

export const txBlobExample =
  '0abe020abb020aa8010a300a0a08c8daccb4a6f185e40612220a2029ea9c2f3371f6a487e7e95c247041f4a356f983eb064e5d2b3bcf322ca96a10122085197c5d60cf28b5ec756a657957b310072396577956fd5cd421ca62b4a6bc091a520a50890bc98e3698aa4578e419b028da5672e627c280d8b06166f4c42d5366bccf1fcf3b296cd61e8d744a21f75f2fb697183e18595d8a79008539d8fb138b405db09db65cc42d54c0e772e5d42d5f20b52f10f1a9e496d5f01d1a20732b53ee807140dd5672768ec1a38be09c531a0c6fc185d5f51c18f5f2261d012220f2e2f45f0ea734d7c11321cbf20427b379cfed6f71874ff97e8bcbbfce2d3d012a2073ec22fcaeccfadc720dd0350cf6af7ec274a74be832e8334613638edfd2fb10322093043bfea2094b0398f0e14bccc66a9ec335bbfd1f8e8b4c2c21428947f5e50d121c08cec08d8e1e1206757673762d361a0c0a0a08d6fab2e5c4f992aa0b'
