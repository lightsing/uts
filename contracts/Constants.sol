// SPDX-License-Identifier: MIT

pragma solidity ^0.8.24;

import {IUniversalTimestamps} from "./core/IUniversalTimestamps.sol";
import {IL1GasPriceOracle} from "scroll-contracts/L2/predeploys/IL1GasPriceOracle.sol";

library Constants {
    IUniversalTimestamps public constant UTS = IUniversalTimestamps(0xdf939C24d9c075862837e3c9EC0cc1feD6376D59);

    IL1GasPriceOracle public constant L1_GAS_PRICE_ORACLE =
        IL1GasPriceOracle(0x5300000000000000000000000000000000000002);
}
