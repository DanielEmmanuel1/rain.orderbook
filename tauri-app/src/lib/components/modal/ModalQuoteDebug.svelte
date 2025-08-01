<script lang="ts">
  import { debugOrderQuote } from '$lib/queries/orderQuote';
  import { queryClient } from '$lib/queries/queryClient';
  import type { RaindexOrder } from '@rainlanguage/orderbook';
  import { createQuery } from '@tanstack/svelte-query';
  import { Alert, Modal } from 'flowbite-svelte';
  import { Refresh, useRaindexClient } from '@rainlanguage/ui-components';
  import EvalResultsTable from '../debug/EvalResultsTable.svelte';
  import { fade } from 'svelte/transition';

  const raindexClient = useRaindexClient();

  export let open: boolean;
  export let order: RaindexOrder;
  export let inputIOIndex: number;
  export let outputIOIndex: number;
  export let pair: string;
  export let blockNumber: bigint | undefined;

  $: debugQuery = createQuery(
    {
      queryKey: [order + pair + blockNumber],
      queryFn: async () => {
        const network = raindexClient.getNetworkByChainId(order.chainId);
        if (network.error) {
          throw new Error(network.error.readableMsg);
        }

        const sgOrder = order.convertToSgOrder();
        if (sgOrder.error) {
          throw new Error(sgOrder.error.readableMsg);
        }

        const result = await debugOrderQuote(
          sgOrder.value,
          network.value.rpcs,
          inputIOIndex,
          outputIOIndex,
          blockNumber ? Number(blockNumber) : undefined,
        );
        return result;
      },
      retry: 0,
      refetchOnWindowFocus: false,
      refetchInterval: false,
      refetchOnMount: true,
    },
    queryClient,
  );
</script>

<Modal title={`Debugging quote for pair ${pair}`} bind:open outsideclose size="lg">
  <div class="flex items-center">
    {#if $debugQuery.data}
      <div class="flex flex-col text-sm">
        <span class="whitespace-nowrap" data-testid="modal-quote-debug-block-number"
          >Block: {blockNumber}</span
        >
      </div>
    {/if}
    <div class="flex w-full items-center justify-end">
      {#if $debugQuery.isLoading || $debugQuery.isFetching}
        <span class="text-sm" transition:fade data-testid="modal-quote-debug-loading-message"
          >Getting quote stack...</span
        >
      {/if}
      <Refresh
        data-testid="refreshButton"
        class="ml-2 h-8 w-5 cursor-pointer text-gray-400 dark:text-gray-400"
        on:click={() => $debugQuery.refetch()}
        spin={$debugQuery.isLoading || $debugQuery.isFetching}
      />
    </div>
  </div>
  {#if $debugQuery.data}
    {#if !!$debugQuery.data[1]}
      <Alert data-testid="modal-quote-debug-error-partial" color="red">{$debugQuery.data[1]}</Alert>
    {/if}
    <EvalResultsTable table={$debugQuery.data[0]} />
  {/if}
  <div class="flex flex-col gap-y-2 text-sm"></div>
</Modal>
