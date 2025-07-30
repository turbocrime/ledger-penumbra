export interface TestCase {
  idx: number
  name: string
  blob: string
  expected_effect_hash: string
  expected_spend_sig: string
  metadata: string[]
}

export interface PR4948TestCase {
  idx: number
  actionTypes: string[]
  name: string
  blob: string
  expected_effect_hash: string
  expected_spend_sigs: string[]
  expected_delegator_vote_sigs: string[]
  expected_lqt_vote_sigs: string[]
  metadata: string[]
}
