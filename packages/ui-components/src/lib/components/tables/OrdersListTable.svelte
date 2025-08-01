<script lang="ts" generics="T">
	import { getNetworkName } from '$lib/utils/getNetworkName';
	import { goto } from '$app/navigation';
	import { DotsVerticalOutline } from 'flowbite-svelte-icons';
	import { createInfiniteQuery, createQuery } from '@tanstack/svelte-query';
	import { RaindexOrder } from '@rainlanguage/orderbook';
	import TanstackAppTable from '../TanstackAppTable.svelte';
	import { formatTimestampSecondsAsLocal } from '../../services/time';
	import ListViewOrderbookFilters from '../ListViewOrderbookFilters.svelte';
	import Hash, { HashType } from '../Hash.svelte';
	import { DEFAULT_PAGE_SIZE, DEFAULT_REFRESH_INTERVAL } from '../../queries/constants';
	import { QKEY_ORDERS, QKEY_TOKENS } from '../../queries/keys';
	import type { AppStoresInterface } from '../../types/appStores';
	import {
		Badge,
		Button,
		Dropdown,
		DropdownItem,
		TableBodyCell,
		TableHeadCell
	} from 'flowbite-svelte';
	import { useAccount } from '$lib/providers/wallet/useAccount';
	import { useRaindexClient } from '$lib/hooks/useRaindexClient';
	import { getAllContexts } from 'svelte';

	const context = getAllContexts();

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	export let handleOrderRemoveModal: any = undefined;
	// End of optional props

	export let selectedChainIds: AppStoresInterface['selectedChainIds'];
	export let activeAccountsItems: AppStoresInterface['activeAccountsItems'] | undefined;
	export let showInactiveOrders: AppStoresInterface['showInactiveOrders'];
	export let orderHash: AppStoresInterface['orderHash'];
	export let hideZeroBalanceVaults: AppStoresInterface['hideZeroBalanceVaults'];
	export let showMyItemsOnly: AppStoresInterface['showMyItemsOnly'];
	export let activeTokens: AppStoresInterface['activeTokens'];

	const { matchesAccount, account } = useAccount();
	const raindexClient = useRaindexClient();

	$: owners =
		$activeAccountsItems && Object.values($activeAccountsItems).length > 0
			? Object.values($activeAccountsItems)
			: $showMyItemsOnly && $account
				? [$account]
				: [];

	$: tokensQuery = createQuery({
		queryKey: [QKEY_TOKENS, $selectedChainIds],
		queryFn: async () => {
			const result = await raindexClient.getAllVaultTokens($selectedChainIds);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		enabled: true
	});

	$: selectedTokens =
		$activeTokens?.filter(
			(address) => !$tokensQuery.data || $tokensQuery.data.some((t) => t.address === address)
		) ?? [];

	$: query = createInfiniteQuery({
		queryKey: [
			QKEY_ORDERS,
			$selectedChainIds,
			owners,
			$showInactiveOrders,
			$orderHash,
			selectedTokens
		],
		queryFn: async ({ pageParam }) => {
			const result = await raindexClient.getOrders(
				$selectedChainIds,
				{
					owners,
					active: $showInactiveOrders ? undefined : true,
					orderHash: $orderHash || undefined,
					tokens: selectedTokens
				},
				pageParam + 1
			);
			if (result.error) throw new Error(result.error.readableMsg);
			return result.value;
		},
		initialPageParam: 0,
		getNextPageParam(lastPage, _allPages, lastPageParam) {
			return lastPage.length === DEFAULT_PAGE_SIZE ? lastPageParam + 1 : undefined;
		},
		refetchInterval: DEFAULT_REFRESH_INTERVAL,
		enabled: true
	});

	const AppTable = TanstackAppTable<RaindexOrder>;
</script>

<ListViewOrderbookFilters
	{selectedChainIds}
	{activeAccountsItems}
	{showMyItemsOnly}
	{showInactiveOrders}
	{orderHash}
	{hideZeroBalanceVaults}
	{tokensQuery}
	{activeTokens}
	{selectedTokens}
/>

<AppTable
	{query}
	queryKey={QKEY_ORDERS}
	emptyMessage="No Orders Found"
	on:clickRow={(e) => {
		goto(`/orders/${e.detail.item.chainId}-${e.detail.item.orderbook}-${e.detail.item.orderHash}`);
	}}
>
	<svelte:fragment slot="title">
		<div class="mt-2 flex w-full justify-between">
			<div class="text-3xl font-medium dark:text-white" data-testid="title">Orders</div>
			<slot name="filters" />
		</div>
	</svelte:fragment>

	<svelte:fragment slot="head">
		<TableHeadCell data-testid="orderListHeadingNetwork" padding="p-4">Network</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingActive" padding="p-4">Active</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingID" padding="p-4">Order</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingOwner" padding="p-4">Owner</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingOrderbook" padding="p-4">Orderbook</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingLastAdded" padding="p-4">Last Added</TableHeadCell>
		<TableHeadCell data-testid="orderListHeadingInputs" padding="px-2 py-4"
			>Input Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingOutputs" padding="px-2 py-4"
			>Output Token(s)</TableHeadCell
		>
		<TableHeadCell data-testid="orderListHeadingTrades" padding="px-2 py-4">Trades</TableHeadCell>
	</svelte:fragment>

	<svelte:fragment slot="bodyRow" let:item>
		<TableBodyCell data-testid="orderListRowNetwork" tdClass="px-4 py-2">
			{getNetworkName(Number(item.chainId))}
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowActive" tdClass="px-4 py-2">
			{#if item.active}
				<Badge color="green">Active</Badge>
			{:else}
				<Badge color="yellow">Inactive</Badge>
			{/if}
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowID" tdClass="break-all px-4 py-4">
			<Hash type={HashType.Identifier} value={item.orderHash} />
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowOwner" tdClass="break-all px-4 py-2">
			<Hash type={HashType.Wallet} value={item.owner} />
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowOrderbook" tdClass="break-all px-4 py-2">
			<Hash type={HashType.Identifier} value={item.orderbook} />
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowLastAdded" tdClass="break-word px-4 py-2">
			{formatTimestampSecondsAsLocal(item.timestampAdded)}
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowInputs" tdClass="break-word p-2">
			{item.inputs.map((t) => t.token.symbol)}
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowOutputs" tdClass="break-word p-2">
			{item.outputs.map((t) => t.token.symbol)}
		</TableBodyCell>
		<TableBodyCell data-testid="orderListRowTrades" tdClass="break-word p-2"
			>{item.tradesCount > 99 ? '>99' : item.tradesCount}</TableBodyCell
		>
		{#if matchesAccount(item.owner) && handleOrderRemoveModal}
			<div data-testid="wallet-actions">
				<TableBodyCell tdClass="px-0 text-right">
					{#if item.active}
						<Button
							color="alternative"
							outline={false}
							data-testid={`order-menu-${item.id}`}
							id={`order-menu-${item.id}`}
							class="mr-2 border-none px-2"
							on:click={(e) => {
								e.stopPropagation();
							}}
						>
							<DotsVerticalOutline class="dark:text-white" />
						</Button>
					{/if}
				</TableBodyCell>
				{#if item.active}
					<Dropdown placement="bottom-end" triggeredBy={`#order-menu-${item.id}`}>
						<DropdownItem
							on:click={(e) => {
								e.stopPropagation();
								handleOrderRemoveModal(item, $query.refetch, context);
							}}>Remove</DropdownItem
						>
					</Dropdown>
				{/if}
			</div>
		{/if}
	</svelte:fragment>
</AppTable>
