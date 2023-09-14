# multi-vault

MultiVault-implementation of a Vault for multiple assets within a NFT collection, with entitlements.
Vault holds a multiple NFT asset in escrow on behalf of multiple beneficial owners. Other contracts
are able to register "entitlements" for a fixed period of time on the asset, which give them the ability to
change the vault's owner.
This contract views the tokenId for the asset on the ERC721 contract as the corresponding assetId for that asset
when deposited into the vault.
