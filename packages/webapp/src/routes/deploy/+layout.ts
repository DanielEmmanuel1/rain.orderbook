import { REGISTRY_URL } from '$lib/constants';
import type { LayoutLoad } from './$types';
import type { InvalidOrderDetail, ValidOrderDetail } from '@rainlanguage/ui-components';
import { fetchRegistryDotrains, validateOrders } from '@rainlanguage/ui-components/services';
import type { RegistryDotrain } from '@rainlanguage/ui-components/services';

/**
+ * Type defining the structure of the load function's return value,
+ * including registry information and validation results.
+ */
type LoadResult = {
	registryFromUrl: string;
	registryDotrains: RegistryDotrain[];
	validOrders: ValidOrderDetail[];
	invalidOrders: InvalidOrderDetail[];
	error: string | null;
};

export const load: LayoutLoad<LoadResult> = async ({ url }) => {
	const registryFromUrl = url.searchParams.get('registry') || REGISTRY_URL;

	try {
		const registryDotrains = await fetchRegistryDotrains(registryFromUrl);

		const { validOrders, invalidOrders } = await validateOrders(registryDotrains);

		return {
			registryFromUrl,
			registryDotrains,
			validOrders,
			invalidOrders,
			error: null
		};
	} catch (error: unknown) {
		return {
			registryFromUrl,
			registryDotrains: [],
			validOrders: [],
			invalidOrders: [],
			error: error instanceof Error ? error.message : 'Unknown error occurred'
		};
	}
};
