# Designity Contract

Anchor-based Solana program that manages organizations, contributor score NFTs, and progression logic. It lets an organization authority mint a collection NFT, register contributors with individual score NFTs, verify collection membership, record scores, and update on-chain metadata as contributors progress.

## Program entrypoints
- `create_organization` — initialize an `Org` account, mint the organization collection NFT, and set scoring config (`weights`, `ranges`, `levels`, `min_reviews`, `domain`, `level_wait`).
- `register` — authority registers an `applicant`, initializes their `Score` PDA, mints their score NFT, and writes initial metadata that includes level URIs under `domain`.
- `verify` — marks a contributor score NFT as a verified member of the organization collection.
- `receive_score` — authority records new review scores, recalculates weighted averages, and (when verified and thresholds met) bumps contributor levels and updates NFT metadata.
- `send_score` — increments `reviews_sent` for bookkeeping when sending out a review request.
- `update_scores` — authority-driven override to set aggregate scores, reviews received, timestamps, and (optionally) force levels, then refresh NFT metadata.

## Accounts & PDAs
- Organization: `seeds = ["org", org_mint, authority]`
- Score: `seeds = ["score", org, applicant]`
- Both `Org` and `Score` support on-demand realloc with rent top-up via `Realloc`.

## File layout
- `programs/designity-contract/src/lib.rs` — program entrypoints.
- `programs/designity-contract/src/instructions/` — handlers for each instruction.
- `programs/designity-contract/src/state/` — `Org` and `Score` account data + scoring logic.
- `programs/designity-contract/src/utils/` — shared realloc helper.
- `migrations/deploy.ts` — Anchor deploy script.
- `tests/` — placeholder for Anchor integration tests.

## Local development
Requirements: `anchor-cli`, `node`, `yarn` (or `npm`), and a running local validator (`solana-test-validator`).

Install deps (workspace root):
```bash
yarn install
cargo build-bpf   # or `anchor build` for the program
```

Run tests:
```bash
anchor test
```

Deploy (uses `Anchor.toml` config):
```bash
anchor deploy
```

## Instruction highlights & parameters
- Scoring configuration:
  - `weights`: weight per dimension.
  - `ranges`: boundaries defining how many averaged groups exist (`ranges.len() + 1 == levels.len()`).
  - `levels`: ascending thresholds per dimension/group.
  - `min_reviews`: minimum reviews required before level ups apply.
  - `level_wait`: cooldown window (seconds) between level changes (enforced in `receive_score`).
- Metadata:
  - Organization NFT uses symbol `GRWTH`, URI `${domain}/org.json`.
  - Contributor score NFTs use symbol `SCORE`, URI `${domain}/{level-string}.json` where `level-string` is hyphen-joined levels.
  - `verify` must be run once per contributor NFT to mark it as part of the collection before metadata updates are allowed.

## Notes and assumptions
- Authority-only flows: `create_organization`, `register`, `receive_score`, `update_scores`, and `verify` all assert the caller is the organization authority.
- `timestamp_override` in `receive_score` lets the authority backfill submissions; pass `0` to default to cluster time.
- Realloc helpers top up rent before resizing PDAs; ensure the payer signer funds have sufficient SOL.

