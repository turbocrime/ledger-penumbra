/*******************************************************************************
*   (c) 2024 Zondax GmbH
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License.
********************************************************************************/

use crate::constants::ACTION_DATA_QTY;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActionPlan {
    Spend = 1,
    Output = 2,
    Swap = 3,
    SwapClaim = 4,
    ValidatorDefinition = 16,
    IbcAction = 17,
    ProposalSubmit = 18,
    ProposalWithdraw = 19,
    ValidatorVote = 20,
    DelegatorVote = 21,
    ProposalDepositClaim = 22,
    PositionOpen = 30,
    PositionClose = 31,
    PositionWithdraw = 32,
    PositionOpenPlan = 35,
    Delegate = 40,
    Undelegate = 41,
    UndelegateClaim = 42,
    CommunityPoolSpend = 50,
    CommunityPoolOutput = 51,
    CommunityPoolDeposit = 52,
    Ics20Withdrawal = 200,
    ActionDutchAuctionSchedule = 53,
    ActionDutchAuctionEnd = 54,
    ActionDutchAuctionWithdraw = 55,
}

impl ActionPlan {
    pub fn from(action_type: u8) -> Self {
        unsafe { std::mem::transmute(action_type) }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ActionHash(pub [u8; 64]);

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(any(feature = "derive-debug", test), derive(Debug))]
pub struct ActionsHashC {
    pub qty: u8,
    pub hashes: [ActionHash; ACTION_DATA_QTY],
}
