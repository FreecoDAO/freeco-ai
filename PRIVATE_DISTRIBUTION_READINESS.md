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
- Use the `Release` workflow only by manual dispatch after entering a counsel
  approval reference and confirming private distribution. Tag pushes no longer
  publish artifacts automatically.
- Store release signing and registry credentials only in protected secrets and
  restrict workflow dispatch to release administrators.

## Entitlement delivery controls

Use a payment and membership system that records the customer legal entity,
paid membership/subscription/lump-sum entitlement, acceptance of the approved
commercial agreement, authorized users, and expiry date. The distribution
service must issue short-lived, authenticated download URLs only after it
checks that record.

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
