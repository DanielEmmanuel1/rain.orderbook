import { vi, describe } from 'vitest';
import Page from './+page.svelte';
import { render, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import {
	useAccount,
	useToasts,
	useTransactions,
	type TransactionConfirmationProps
} from '@rainlanguage/ui-components';
import { readable, writable } from 'svelte/store';
import { DotrainOrderGui } from '@rainlanguage/orderbook';
import { REGISTRY_URL } from '$lib/constants';
import { handleTransactionConfirmationModal } from '$lib/services/modal';

const { mockPageStore, mockSettingsStore } = await vi.hoisted(
	() => import('@rainlanguage/ui-components')
);

const { mockConnectedStore, mockAppKitModalStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('$lib/__mocks__/stores')
);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		useTransactions: vi.fn(),
		useAccount: vi.fn(),
		useToasts: vi.fn()
	};
});

vi.mock('$lib/services/modal', async () => ({
	handleDisclaimerModal: vi.fn(async (props) => {
		const { DisclaimerModal } = await import('@rainlanguage/ui-components');
		new DisclaimerModal({
			target: document.body,
			props: { ...props, open: true }
		});
	}),
	handleTransactionConfirmationModal: vi.fn()
}));

vi.mock('$app/stores', async (importOriginal) => {
	return {
		...((await importOriginal()) as object),
		page: mockPageStore
	};
});

vi.mock('$lib/stores/wagmi', () => ({
	connected: mockConnectedStore,
	appKitModal: mockAppKitModalStore,
	wagmiConfig: mockWagmiConfigStore
}));

describe('Full Deployment Tests', () => {
	let fixedLimitStrategy: string;
	let auctionStrategy: string;
	let dynamicSpreadStrategy: string;

	const fetchRegistry = async () => {
		const response = await fetch(REGISTRY_URL);
		const registry = await response.text();
		const linksMap = Object.fromEntries(
			registry
				.split('\n')
				.map((line) => line.trim().split(' '))
				.filter((parts) => parts.length === 2)
		);
		return linksMap;
	};
	const fetchStrategy = async (url: string) => {
		try {
			const response = await fetch(url);
			return await response.text();
		} catch (error) {
			assert.fail(error as string);
		}
	};
	function findLockRegion(a: string, b: string): { prefixEnd: number; suffixEnd: number } {
		expect(a.length).toEqual(b.length);
		const length = a.length;
		// Find prefix end
		let prefixEnd = 0;
		while (prefixEnd < length && a[prefixEnd] === b[prefixEnd]) {
			prefixEnd++;
		}
		// Find suffix start
		let suffixEnd = length;
		while (suffixEnd > prefixEnd && a[suffixEnd - 1] === b[suffixEnd - 1]) {
			suffixEnd--;
		}
		return { prefixEnd, suffixEnd };
	}

	beforeAll(async () => {
		const registry = await fetchRegistry();
		fixedLimitStrategy = await fetchStrategy(registry['fixed-limit']);
		assert(fixedLimitStrategy, 'Fixed limit strategy not found');
		auctionStrategy = await fetchStrategy(registry['auction-dca']);
		assert(auctionStrategy, 'Auction strategy not found');
		dynamicSpreadStrategy = await fetchStrategy(registry['dynamic-spread']);
		assert(dynamicSpreadStrategy, 'Dynamic spread strategy not found');
	});

	beforeEach(async () => {
		vi.clearAllMocks();
		vi.mocked(useAccount).mockReturnValue({
			account: readable('0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'),
			matchesAccount: vi.fn()
		});
		vi.mocked(useToasts).mockReturnValue({
			removeToast: vi.fn(),
			toasts: writable([]),
			addToast: vi.fn(),
			errToast: vi.fn()
		});
		vi.mocked(useTransactions).mockReturnValue({
			// @ts-expect-error simple object
			manager: writable({}),
			transactions: readable()
		});
		mockConnectedStore.mockSetSubscribeValue(true);
	});

	afterEach(async () => {
		await new Promise((resolve) => setTimeout(resolve, 5000));
	});

	it(
		'Fixed limit strategy',
		async () => {
			mockPageStore.mockSetSubscribeValue({
				data: {
					stores: { settings: mockSettingsStore },
					dotrain: fixedLimitStrategy,
					deployment: {
						key: 'flare'
					},
					strategyDetail: {
						name: 'Fixed limit'
					}
				}
			});
			const screen = render(Page);

			// Wait for the gui provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
				},
				{ timeout: 10000 }
			);

			await waitFor(() => {
				expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
			});
			const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

			await userEvent.click(tokenSelectionButtons[0]);
			await userEvent.click(screen.getByText('Staked FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
			});

			await userEvent.click(tokenSelectionButtons[1]);
			await userEvent.click(screen.getByText('Wrapped FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
			});

			// Get the input component and write "10" into it
			const customValueInput = screen.getAllByPlaceholderText('Enter custom value')[0];
			await userEvent.clear(customValueInput);
			await userEvent.type(customValueInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for token2
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '0x123');

			// Set vault id for token1
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '0x234');

			// Click the "Deploy Strategy" button
			const deployButton = screen.getByText('Deploy Strategy');
			await userEvent.click(deployButton);

			await waitFor(async () => {
				const disclaimerButton = screen.getByText('Deploy');
				await userEvent.click(disclaimerButton);
			});

			const getDeploymentArgs = async () => {
				const gui = (await DotrainOrderGui.newWithDeployment(fixedLimitStrategy, 'flare'))
					.value as DotrainOrderGui;
				await gui.saveSelectToken('token1', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
				await gui.saveSelectToken('token2', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
				gui.setVaultId(false, 0, '0x123');
				gui.setVaultId(true, 0, '0x234');
				gui.saveFieldValue('fixed-io', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Fixed limit strategy error', error);
				return null;
			});

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock
				.calls[0][0] as TransactionConfirmationProps;

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 30000 }
	);

	it(
		'Auction strategy',
		async () => {
			mockPageStore.mockSetSubscribeValue({
				data: {
					stores: { settings: mockSettingsStore },
					dotrain: auctionStrategy,
					deployment: {
						key: 'flare'
					},
					strategyDetail: {
						name: 'Auction'
					}
				}
			});

			const screen = render(Page);

			// Wait for the gui provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
				},
				{ timeout: 10000 }
			);

			// Check that the token dropdowns are present
			await waitFor(() => {
				expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
			});
			const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

			await userEvent.click(tokenSelectionButtons[0]);
			await userEvent.click(screen.getByText('Staked FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-output')).toBeInTheDocument();
			});

			await userEvent.click(tokenSelectionButtons[1]);
			await userEvent.click(screen.getByText('Wrapped FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-input')).toBeInTheDocument();
			});

			const timePerAmountEpochInput = screen.getByTestId(
				'binding-time-per-amount-epoch-input'
			) as HTMLInputElement;
			await userEvent.clear(timePerAmountEpochInput);
			await userEvent.type(timePerAmountEpochInput, '60');

			const amountPerEpochInput = screen.getByTestId(
				'binding-amount-per-epoch-input'
			) as HTMLInputElement;
			await userEvent.clear(amountPerEpochInput);
			await userEvent.type(amountPerEpochInput, '10');

			const maxTradeAmountInput = screen.getByTestId(
				'binding-max-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(maxTradeAmountInput);
			await userEvent.type(maxTradeAmountInput, '100');

			const minTradeAmountInput = screen.getByTestId(
				'binding-min-trade-amount-input'
			) as HTMLInputElement;
			await userEvent.clear(minTradeAmountInput);
			await userEvent.type(minTradeAmountInput, '1');

			const baselineInput = screen.getByTestId('binding-baseline-input') as HTMLInputElement;
			await userEvent.clear(baselineInput);
			await userEvent.type(baselineInput, '10');

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for output
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '0x123');

			// Set vault id for input
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '0x234');

			// Click the "Deploy Strategy" button
			const deployButton = screen.getByText('Deploy Strategy');
			await userEvent.click(deployButton);

			await waitFor(async () => {
				const disclaimerButton = screen.getByText('Deploy');
				await userEvent.click(disclaimerButton);
			});

			const getDeploymentArgs = async () => {
				const gui = (await DotrainOrderGui.newWithDeployment(auctionStrategy, 'flare'))
					.value as DotrainOrderGui;
				await gui.saveSelectToken('input', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
				await gui.saveSelectToken('output', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
				gui.setVaultId(false, 0, '0x123');
				gui.setVaultId(true, 0, '0x234');
				gui.saveFieldValue('time-per-amount-epoch', '60');
				gui.saveFieldValue('amount-per-epoch', '10');
				gui.saveFieldValue('max-trade-amount', '100');
				gui.saveFieldValue('min-trade-amount', '1');
				gui.saveFieldValue('baseline', '10');
				gui.saveFieldValue('initial-io', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Auction strategy error', error);
				return null;
			});

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock
				.calls[0][0] as TransactionConfirmationProps;

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 30000 }
	);

	it.only(
		'Dynamic spread strategy',
		async () => {
			mockPageStore.mockSetSubscribeValue({
				data: {
					stores: { settings: mockSettingsStore },
					dotrain: dynamicSpreadStrategy,
					deployment: {
						key: 'flare'
					},
					strategyDetail: {
						name: 'Dynamic spread'
					}
				}
			});

			const screen = render(Page);

			// Wait for the gui provider to be in the document
			await waitFor(
				() => {
					expect(screen.getByTestId('gui-provider')).toBeInTheDocument();
				},
				{ timeout: 10000 }
			);

			await waitFor(() => {
				expect(screen.getAllByRole('button', { name: /chevron down solid/i }).length).toBe(2);
			});
			const tokenSelectionButtons = screen.getAllByRole('button', { name: /chevron down solid/i });

			await userEvent.click(tokenSelectionButtons[0]);
			await userEvent.click(screen.getByText('Staked FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token1')).toBeInTheDocument();
			});

			await userEvent.click(tokenSelectionButtons[1]);
			await userEvent.click(screen.getByText('Wrapped FLR'));
			await waitFor(() => {
				expect(screen.getByTestId('select-token-success-token2')).toBeInTheDocument();
			});

			const amountIsFastExitButton = screen.getByTestId(
				'binding-amount-is-fast-exit-preset-Yes'
			) as HTMLElement;
			await userEvent.click(amountIsFastExitButton);

			const notAmountIsFastExitButton = screen.getByTestId(
				'binding-not-amount-is-fast-exit-preset-No'
			) as HTMLElement;
			await userEvent.click(notAmountIsFastExitButton);

			const initialIoInput = screen.getByTestId('binding-initial-io-input') as HTMLInputElement;
			await userEvent.clear(initialIoInput);
			await userEvent.type(initialIoInput, '100');

			const maxAmountInput = screen.getByTestId('binding-max-amount-input') as HTMLInputElement;
			await userEvent.clear(maxAmountInput);
			await userEvent.type(maxAmountInput, '1000');

			const minAmountInput = screen.getByTestId('binding-min-amount-input') as HTMLInputElement;
			await userEvent.clear(minAmountInput);
			await userEvent.type(minAmountInput, '10');

			const showAdvancedOptionsButton = screen.getByText('Show advanced options');
			await userEvent.click(showAdvancedOptionsButton);

			const vaultIdInputs = screen.getAllByTestId('vault-id-input') as HTMLInputElement[];

			// Set vault id for token1
			await userEvent.clear(vaultIdInputs[0]);
			await userEvent.type(vaultIdInputs[0], '0x234');

			// Set vault id for token2
			await userEvent.clear(vaultIdInputs[1]);
			await userEvent.type(vaultIdInputs[1], '0x123');

			// Click the "Deploy Strategy" button
			const deployButton = screen.getByText('Deploy Strategy');
			await userEvent.click(deployButton);

			await waitFor(async () => {
				const disclaimerButton = screen.getByText('Deploy');
				await userEvent.click(disclaimerButton);
			});

			const getDeploymentArgs = async () => {
				const gui = (await DotrainOrderGui.newWithDeployment(dynamicSpreadStrategy, 'flare'))
					.value as DotrainOrderGui;
				await gui.saveSelectToken('token1', '0x1D80c49BbBCd1C0911346656B529DF9E5c2F783d');
				await gui.saveSelectToken('token2', '0x12e605bc104e93B45e1aD99F9e555f659051c2BB');
				gui.setVaultId(false, 0, '0x123');
				gui.setVaultId(true, 0, '0x234');
				gui.saveFieldValue('amount-is-fast-exit', '1');
				gui.saveFieldValue('not-amount-is-fast-exit', '0');
				gui.saveFieldValue('initial-io', '100');
				gui.saveFieldValue('max-amount', '1000');
				gui.saveFieldValue('min-amount', '10');
				const args = await gui.getDeploymentTransactionArgs(
					'0x999999cf1046e68e36E1aA2E0E07105eDDD1f08E'
				);
				return args.value;
			};
			await new Promise((resolve) => setTimeout(resolve, 5000));
			const args = await getDeploymentArgs().catch((error) => {
				// eslint-disable-next-line no-console
				console.log('Dynamic spread strategy error', error);
				return null;
			});

			// @ts-expect-error mock is not typed
			const callArgs = handleTransactionConfirmationModal.mock
				.calls[0][0] as TransactionConfirmationProps;

			const { prefixEnd, suffixEnd } = findLockRegion(
				callArgs.args.calldata,
				args?.deploymentCalldata || ''
			);

			expect(callArgs.args.calldata.length).toEqual(args?.deploymentCalldata.length);
			expect(callArgs.args.calldata.slice(0, prefixEnd)).toEqual(
				args?.deploymentCalldata.slice(0, prefixEnd)
			);
			// The middle section of the calldata is different because of secret and nonce
			expect(callArgs.args.calldata.slice(prefixEnd, suffixEnd)).not.toEqual(
				args?.deploymentCalldata.slice(prefixEnd, suffixEnd)
			);
			expect(callArgs.args.calldata.slice(suffixEnd)).toEqual(
				args?.deploymentCalldata.slice(suffixEnd)
			);
			expect(callArgs.args.toAddress).toEqual(args?.orderbookAddress);
			expect(callArgs.args.chainId).toEqual(args?.chainId);
		},
		{ timeout: 30000 }
	);
});
