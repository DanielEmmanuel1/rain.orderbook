// SPDX-License-Identifier: LicenseRef-DCL-1.0
// SPDX-FileCopyrightText: Copyright (c) 2020 Rain Open Source Software Ltd
pragma solidity =0.8.25;

import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {
    OrderConfigV4,
    OrderV4,
    EvaluableV4,
    ClearConfigV2,
    SignedContextV1,
    TaskV2
} from "rain.orderbook.interface/interface/unstable/IOrderBookV5.sol";
import {UnsupportedCalculateOutputs} from "src/concrete/ob/OrderBook.sol";
import {IERC20Metadata} from "openzeppelin-contracts/contracts/token/ERC20/extensions/IERC20Metadata.sol";

contract OrderBookClearOrderBadStackTest is OrderBookExternalRealTest {
    function checkBadStack(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob,
        bytes memory rainStringAlice,
        bytes memory rainStringBob,
        uint256 badStackHeight
    ) internal {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(configAlice, iInterpreter, iStore);
        LibTestAddOrder.conformConfig(configBob, iInterpreter, iStore);
        configAlice.validInputs[0].token = address(iToken0);
        configAlice.validOutputs[0].token = address(iToken1);
        configBob.validOutputs[0] = configAlice.validInputs[0];
        configBob.validInputs[0] = configAlice.validOutputs[0];

        configAlice.evaluable.bytecode = iParserV2.parse2(rainStringAlice);
        configBob.evaluable.bytecode = iParserV2.parse2(rainStringBob);

        OrderV4 memory orderAlice =
            OrderV4(alice, configAlice.evaluable, configAlice.validInputs, configAlice.validOutputs, configAlice.nonce);
        OrderV4 memory orderBob =
            OrderV4(bob, configBob.evaluable, configBob.validInputs, configBob.validOutputs, configBob.nonce);

        vm.prank(alice);
        iOrderbook.addOrder3(configAlice, new TaskV2[](0));

        vm.prank(bob);
        iOrderbook.addOrder3(configBob, new TaskV2[](0));

        vm.expectRevert(abi.encodeWithSelector(UnsupportedCalculateOutputs.selector, badStackHeight));
        iOrderbook.clear3(
            orderAlice, orderBob, ClearConfigV2(0, 0, 0, 0, 0, 0), new SignedContextV1[](0), new SignedContextV1[](0)
        );
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackEmptyStack(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, ":;:;", ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneStack(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_:1;:;", "_:1;:;", 1);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneEmpty(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_ _:1 1;:;", ":;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOtherEmpty(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, ":;:;", "_ _:1 1;:;", 0);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneOne(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_ _:1 1;:;", "_:1;:;", 1);
    }

    /// forge-config: default.fuzz.runs = 100
    function testClearOrderBadStackOneOtherOne(
        address alice,
        address bob,
        OrderConfigV4 memory configAlice,
        OrderConfigV4 memory configBob
    ) external {
        checkBadStack(alice, bob, configAlice, configBob, "_:1;:;", "_ _:1 1;:;", 1);
    }
}
