# Private Distribution Readiness

This repository cannot itself verify memberships, payments, or customer
eligibility. Before a commercial distribution, an authorized Free Eco
Association administrator must complete and record the following controls.

## GitHub and artifact controls

- Set the repository to **private** and grant repository and organization
  access only to authorized Association personnel.
- Set the `freeco-ai` GHCR package to **private** and confirm no public
  package, GitHub release, source archive, container tag, mirror, or download
  URL remains available.
- Set `PRIVATE_DISTRIBUTION_READY=true` and a non-empty
  `COUNSEL_APPROVAL_REFERENCE` repository variable only after the authorized
  administrator and counsel complete the required review. Merging a PR labelled
  `release` is then the only release entry point: it creates the metadata commit
  and tag, and the tag triggers the artifact build automatically.
- Store release signing and registry credentials only in protected secrets and
  restrict workflow dispatch to release administrators.

## Entitlement delivery controls

Use a payment and membership system that records the customer legal entity,
paid membership/subscription/lump-sum entitlement, acceptance of the approved
commercial agreement, authorized users, and expiry date. The distribution
service must issue short-lived, authenticated download URLs only after it
checks that record.

The CLI installers and desktop updater can send `FREECO_AI_RELEASE_TOKEN` to
an authorized private GitHub release. Treat that only as a compatibility path
for authorized repository access: never embed a long-lived token in a binary,
installer URL, or customer-facing document, and do not use it instead of the
entitlement service.

On cancellation, expiry, failed payment, or eligibility revocation, revoke
download access and service credentials promptly. Do not put payment records,
identity documents, license keys, or signed agreements in this repository.

## Required audit before release

1. Inventory each shipped file, binary, dependency, contributor, and source
   provenance.
2. Preserve the associated notices and licenses, including
   `LICENSE-APACHE` and `LICENSE-MIT`.
3. Identify only the Free Eco-owned deliverables that the commercial agreement
   covers; do not characterize inherited or third-party material as exclusive.
4. Record the repository, release, package, and externally hosted download
   visibility review.
5. Have qualified counsel approve the commercial license, eligibility policy,
   contributor terms, privacy terms, and the release-specific attribution
   inventory.

## Release validation

Test with an authorized account and an unauthorized account. Verify that the
authorized account can accept terms and download the entitled artifact, while
the unauthorized account cannot enumerate or download artifacts. Then revoke
the authorized account and verify that new download URLs and service access
are denied. Record the results with the counsel approval reference.
