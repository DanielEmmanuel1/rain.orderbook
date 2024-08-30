import { render, screen, waitFor } from '@testing-library/svelte';
import { test } from 'vitest';
import { expect } from '$lib/test/matchers';
import { mockIPC } from '@tauri-apps/api/mocks';
import ModalTradeDebug from './ModalTradeDebug.svelte';
import { QueryClient } from '@tanstack/svelte-query';
import { mockTradeDebug } from '$lib/queries/tradeDebug';
import { formatEther } from 'viem';

test('renders table with the correct data', async () => {
  const queryClient = new QueryClient();

  mockIPC((cmd) => {
    if (cmd === 'debug_trade') {
      return mockTradeDebug;
    }
  });

  render(ModalTradeDebug, {
    context: new Map([['$$_queryClient', queryClient]]),
    props: { open: true, txHash: '0x123', rpcUrl: 'https://rpc-url.com' },
  });

  expect(await screen.findByText('Debug trade')).toBeInTheDocument();
  expect(await screen.findByTestId('modal-trade-debug-loading-message')).toBeInTheDocument();

  await waitFor(() => {
    expect(screen.queryByTestId('modal-trade-debug-tx-hash')).toHaveTextContent(
      'Trade transaction: 0x123',
    );
    expect(screen.queryByTestId('modal-trade-debug-rpc-url')).toHaveTextContent(
      'RPC: https://rpc-url.com',
    );
  });

  const stacks = await screen.findAllByTestId('modal-trade-debug-stack');
  expect(stacks).toHaveLength(3);
  const values = await screen.findAllByTestId('modal-trade-debug-value');
  expect(values).toHaveLength(3);
  const hexValues = await screen.findAllByTestId('modal-trade-debug-value-hex');
  for (let i = 0; i < 3; i++) {
    expect(stacks[i]).toHaveTextContent(i.toString());
    expect(values[i]).toHaveTextContent(formatEther(BigInt(mockTradeDebug[i])));
    expect(hexValues[i]).toHaveTextContent(mockTradeDebug[i]);
  }
});
