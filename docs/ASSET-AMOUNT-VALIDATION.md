# Exact asset-amount validation

The `AssetAmount` primitive stores nonnegative `i128` integer minor units, not floating-point values. All parsing uses canonical decimal strings without locale separators, exponents, negative signs or silent rounding. Checked arithmetic and ordering require identical asset identities (network, identifier and decimals).

For an explicitly identified classic asset (`classic:` prefix), the representation is bounded to the Stellar signed 64-bit minor-unit maximum; other assets use `i128`. Classic assets should use seven decimals, while Soroban token precision must be supplied from verified contract metadata, never assumed. Asset names and precision must be independently authenticated by higher layers; a valid identifier string does not authenticate a token.

This module neither authorizes a payment nor changes API endpoints. Presentation formatting and a successful parse are not proof of a supported issuer, quote, payout corridor or contract deployment.
