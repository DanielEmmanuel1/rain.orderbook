import { Bytes, ethereum, crypto } from "@graphprotocol/graph-ts";
import { Trade, TradeVaultBalanceChange } from "../generated/schema";
import { eventId } from "./interfaces/event";

export function makeTradeId(event: ethereum.Event, orderHash: Bytes): Bytes {
  let bytes = eventId(event).concat(orderHash);
  return Bytes.fromByteArray(crypto.keccak256(bytes));
}

export function createTradeEntity(
  event: ethereum.Event,
  orderHash: Bytes,
  inputVaultBalanceChange: TradeVaultBalanceChange,
  outputVaultBalanceChange: TradeVaultBalanceChange
): void {
  let trade = new Trade(makeTradeId(event, orderHash));
  trade.orderbook = event.address;
  trade.order = orderHash;
  trade.inputVaultBalanceChange = inputVaultBalanceChange.id;
  trade.outputVaultBalanceChange = outputVaultBalanceChange.id;
  trade.tradeEvent = eventId(event);
  trade.timestamp = event.block.timestamp;
  trade.save();
}
