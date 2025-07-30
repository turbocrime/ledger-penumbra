#!/usr/bin/env node

import { TransactionPlan } from '@penumbra-zone/protobuf/penumbra/core/transaction/v1/transaction_pb';
import { jsonOptions } from '@penumbra-zone/protobuf';
import { authorizePlan } from '@penumbra-zone/wasm/build';
import { generateSpendKey } from '@penumbra-zone/wasm/keys';
import fs from 'fs';

const PR_COMMIT = 'd2e56dbf6c721784056d88740b072c3c194cf6eb';
const TEST_SEED = "equip will roof matter pink blind book anxiety banner elbow sun young";

async function fetchJson(url) {
  const response = await fetch(url);
  return response.ok ? response.json() : null;
}


function convertPlanToTestCase(jsonPlan, index) {
  const plan = TransactionPlan.fromJson(jsonPlan, jsonOptions);
  const hexBlob = Buffer.from(plan.toBinary()).toString('hex');
  
  const spendKey = generateSpendKey(TEST_SEED);
  const authData = authorizePlan(spendKey, plan);
  
  const generatedEffectHash = Buffer.from(authData.effectHash.inner).toString('hex');
  
  const toHexArray = (auths) => auths?.map(sig => Buffer.from(sig.toBinary()).toString('hex')) || [];
  
  // Extract action types from the transaction plan
  const actionTypes = plan.actions.map(action => action.action.case);
  
  return {
    idx: index,
    name: `Penumbra_PR4948_Vector_${index}`,
    blob: hexBlob,
    expected_effect_hash: generatedEffectHash,
    expected_spend_sigs: toHexArray(authData?.spendAuths),
    expected_delegator_vote_sigs: toHexArray(authData?.delegatorVoteAuths),
    expected_lqt_vote_sigs: toHexArray(authData?.lqtVoteAuths),
    actionTypes: actionTypes,
    metadata: []
  };
}

async function main() {
  const indices = Array.from({length: 11}, (_, i) => i);
  
  const plans = await Promise.all(indices.map(async i => {
    if (i === 2) {
      return null;
    }
    const planData = await fetchJson(`https://raw.githubusercontent.com/penumbra-zone/penumbra/${PR_COMMIT}/crates/core/transaction/tests/signing_test_vectors/transaction_plan_${i}.json`);
    
    return planData ? { index: i, plan: planData } : null;
  }));
  
  const validPlans = plans.filter(Boolean);
  const testCases = validPlans.map(({plan, index}) => 
    convertPlanToTestCase(plan, index)
  );
  
  fs.writeFileSync('./penumbra_pr4948_action_testcases.json', JSON.stringify(testCases, null, 2));
  fs.writeFileSync('./penumbra_pr4948_transaction_plans.json', JSON.stringify(validPlans, null, 2));
  
  console.log(`Generated ${testCases.length} test cases`);
}

main().catch(console.error);