import { parseYaml } from '@rainlanguage/orderbook';

const yamlContent = [
	`version: 1
    networks:
      flare:
        key: flare
        rpc: https://rpc.ankr.com/flare
        chainId: 14
        currency: FLR
      base:
        key: base
        rpc: https://mainnet.base.org
        chainId: 8453
        currency: ETH
      sepolia:
        key: sepolia
        rpc: https://1rpc.io/sepolia
        chainId: 11155111
        currency: ETH
      polygon:
        key: polygon
        rpc: https://rpc.ankr.com/polygon
        chainId: 137
        currency: POL
      arbitrum:
        key: arbitrum
        rpc: https://rpc.ankr.com/arbitrum
        chainId: 42161
        currency: ETH
      matchain:
        key: matchain
        rpc: https://rpc.ankr.com/polygon
        chainId: 137
        currency: POL
      bsc:
        key: bsc
        rpc: https://rpc.ankr.com/bsc
        chainId: 56
        currency: BNB
      linea:
        key: linea
        rpc: https://rpc.linea.build
        chainId: 59144
        currency: ETH
      ethereum:
        key: ethereum
        rpc: https://rpc.ankr.com/eth
        chainId: 1
        currency: ETH
    subgraphs:
      flare:
        key: flare
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-flare/0.8/gn
      base:
        key: base
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-base/0.9/gn
      sepolia:
        key: sepolia
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-sepolia/0.1/gn
      arbitrum:
        key: arbitrum
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-arbitrum/0.2/gn
      bsc:
        key: bsc
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-bsc/2024-10-14-63f4/gn
      linea:
        key: linea
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-linea/2024-10-14-12fc/gn
      ethereum:
        key: ethereum
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/ob4-mainnet/2024-10-25-af6a/gn
    metaboards:
      flare:
        key: flare
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-flare-0x893BBFB7/0.1/gn
      base:
        key: base
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-base-0x59401C93/0.1/gn
      sepolia:
        key: sepolia
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-sepolia-0x77991674/0.1/gn
      polygon:
        key: polygon
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-polygon/0.1/gn
      arbitrum:
        key: arbitrum
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-arbitrum/0.1/gn
      bsc:
        key: bsc
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-bsc/0.1/gn
      linea:
        key: linea
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/mb-linea-0xed7d6156/1.0.0/gn
      ethereum:
        key: ethereum
        url: https://api.goldsky.com/api/public/project_clv14x04y9kzi01saerx7bxpg/subgraphs/metadata-mainnet/2024-10-25-2857/gn
    orderbooks:
      flare:
        key: flare
        address: 0xCEe8Cd002F151A536394E564b84076c41bBBcD4d
        network: flare
        subgraph: flare
      base:
        key: base
        address: 0xd2938e7c9fe3597f78832ce780feb61945c377d7
        network: base
        subgraph: base
      sepolia:
        key: sepolia
        address: 0x0bB72B4C7c0d47b2CaED07c804D9243C1B8a0728
        network: sepolia
        subgraph: sepolia
      polygon:
        key: polygon
        address: 0x7D2f700b1f6FD75734824EA4578960747bdF269A
        network: polygon
        subgraph: polygon
      arbitrum:
        key: arbitrum
        address: 0x550878091b2B1506069F61ae59e3A5484Bca9166
        network: arbitrum
        subgraph: arbitrum
      matchain:
        key: matchain
        address: 0x40312edab8fe65091354172ad79e9459f21094e2
        network: matchain
        subgraph: matchain
      bsc:
        key: bsc
        address: 0xd2938E7c9fe3597F78832CE780Feb61945c377d7
        network: bsc
        subgraph: bsc
      linea:
        key: linea
        address: 0x22410e2a46261a1B1e3899a072f303022801C764
        network: linea
        subgraph: linea
      ethereum:
        key: ethereum
        address: 0x0eA6d458488d1cf51695e1D6e4744e6FB715d37C
        network: ethereum
        subgraph: ethereum
    deployers:
      flare:
        key: flare
        address: 0xE3989Ea7486c0F418C764e6c511e86f6E8830FAb
        network: flare
      base:
        key: base
        address: 0xC1A14cE2fd58A3A2f99deCb8eDd866204eE07f8D
        network: base
      sepolia:
        key: sepolia
        address: 0x7692BA8446Bb8B3140A2c02df073080BeD0a7F8E
        network: sepolia
      polygon:
        key: polygon
        address: 0xE7116BC05C8afe25e5B54b813A74F916B5D42aB1
        network: polygon
      arbitrum:
        key: arbitrum
        address: 0x9B0D254bd858208074De3d2DaF5af11b3D2F377F
        network: arbitrum
      matchain:
        key: matchain
        address: 0x582d9e838FE6cD9F8147C66A8f56A3FBE513a6A2
        network: matchain
      bsc:
        key: bsc
        address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
        network: bsc
      linea:
        key: linea
        address: 0xA2f56F8F74B7d04d61f281BE6576b6155581dcBA
        network: linea
      ethereum:
        key: ethereum
        address: 0xd19581a021f4704ad4eBfF68258e7A0a9DB1CD77
    accounts:
      wallet1:
        key: wallet1
        address: 0xf08bCbce72f62c95Dcb7c07dCb5Ed26ACfCfBc11
      wallet2:
        key: wallet2
        address: 0x6bc14a99ccf7f9037c98d75eec6c6d807f9d953f
`
];

describe('parseYaml', () => {
	it('should parse YAML', () => {
		const result = parseYaml(yamlContent);
		if (result.error) {
			throw new Error(`Failed to parse YAML in standalone test file: ${result.error.readableMsg}`);
		}
	});
});
