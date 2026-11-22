# prism/identity/4

A Prism id is stitch's Prism id ([afflom/stitch](https://github.com/afflom/stitch),
`src/prism-id.js` and `docs/prism-id.md`). It is pinned by commit in
[stitch.lock](stitch.lock) and copied, with the stitch modules it imports and
the model decisions they call, into [prism-id.mjs](prism-id.mjs) by
[stitch-sync.mjs](stitch-sync.mjs). This file says how the frame uses it.
Stitch's documents are the contract for the id itself, and every
authorization decision is taken by its `Stitch.PrismId` model. When stitch's
Prism model is refined, re-pin:

```sh
node identity/stitch-sync.mjs <stitch commit>   # writes prism-id.mjs and stitch.lock
npm run test:identity                           # the lock matches, every check refuses, the flow signs in
```

prism/identity/4 replaces identity/3 (a Prism id page at id.uor.foundation
and a registry mailbox) and identity/1 and /2 (passkeys). Nothing here runs
on a server of Prism's own. A session has the same shape as in identity/3.

## Parties

| Party | Where | Holds |
| --- | --- | --- |
| Frame | `<prism-frame>` in every Prism app | This origin's device of the id (keys, keyring and log, in IndexedDB `prism-id`), and this app's session key with the id's signed session (IndexedDB `prism-identity`). It checks every signature itself. |
| stitch's relay | The frame's `PRISM_RELAY` (`https://ntfy.sh`), with stitch's relay framing, through Prism id's `createRelayBus` | Messages under names hashed from secrets only holders know: sealed sync messages (`prism-sync:` topics) and link requests and answers (`prism-rendezvous:` topics). It holds them for the warm lifetime (30 minutes). |

A page's `connect-src` must name the relay and its `wss:` origin. The page
build reads `PRISM_RELAY` from the live frame. A page that does not admit the
relay keeps the id on its device: creating, signing in and recovery work, but
linking and sync do not.

## The session, made without asking

There is no sign-in. On the visitor's first touch of the page (or reach for
the icon), the frame makes this origin's device and a new id if it has none,
then this app's session if it has none or it expired. A new id has no other
device and nothing about it is on the relay, so making it sends and asks the
relay nothing (steps 1 and 3 skip the relay); it first goes to the relay when
it links, removes a device or is recovered.

1. The frame takes in the sync messages the relay holds for every boundary
   key it has, and the id's public log, so a device removed elsewhere stops
   here (`canSign` is false: it drops the session, forgets the device and
   makes a new id). It checks this once per page load even with a session.
2. It makes a non-extractable P-256 session key and signs
   `signAction(device, "prism/session/1", {sessionKey, expires}, {audience: location.origin})`.
   `expires` is 7 days away; the most allowed is 8.
3. It checks the result with `verifySession`, as anyone the app shows a
   signature to does: the log resolves for the envelope's prism id, the
   signer is an active device, the audience is this origin, the action is
   fresh (`Stitch.PrismId.actionFresh`, `actionAllowed`). Only then does it
   keep the session.

A kept session is `{app, envelope, records}`. `frame.sign(bytes)` signs with
the session key and returns the signature with it. Anyone can check it later
with `verifySession(session, {fresh: 0})`, as of its signing and bounded by
its expiry.

## A new id

`createPrism`, then `createRecovery(session, {threshold: 1, guardians: [{paper: true}]})`.
The recovery kit is `{kind: "prism/recovery-kit/1", prism, share, records}`,
with the paper share and the log. It waits in this origin's `prism-id` store
(key `kit`) until the visitor saves it from the panel, as JSON.

## Linking

Stitch's out-of-band link, over the relay:

1. Signed-in device: `createLinkOffer`. The panel shows the `prism-link:`
   URI and its QR code (`linkQr`), and subscribes to `rendezvousTopic(secret)`.
2. New device: `parseLinkUri`, a new device, then `requestLink`, published on
   the rendezvous topic. It shows `linkCode(secret, device)`.
3. The person types those six digits into the first device, which calls
   `acceptLinkRequest` with them as `confirmCode` and publishes the answer.
   A wrong code is refused and the offer stays usable until it expires.
4. New device: `completeLink`, then its session. The id it made on its own is dropped.

## Recovery

From a kit: `recoverPrism({prism, records, shares: [share], device, rotateRecovery: {threshold: 1, guardians: [{paper: true}]}})`
binds this browser's new device and revokes every other device. It replaces
the recovery device, so the new kit retires the old one. The frame then takes
in what the relay holds. A used kit's recovery device is retired, and its
retirement kept none of what this replay signed, so this device cannot sign
(or the two retirements fork the log, which resolves for no one): the kit was
already used and is refused, before anything is kept or published. A device
checks its own log with `heardLog(..., {own: true})`, which refuses a second
retirement of a retired recovery device (stitch's `admitRecords`), so a fork
heard on the relay never reaches it.

## Guardians

A guardian is a device of someone else's id that keeps one share of this id's
recovery device. The two meet on `rendezvousTopic(secret)`, the secret in a
link one sends the other (`prism-guardian:` or `prism-help:` and 43
base64url characters); the relay sees neither the secret nor the shares,
which are wrapped to one device each.

1. **Be someone's guardian**: the guardian publishes
   `{kind: "prism/guardian-offer/1", device}` (its device record) and shows the link.
2. **Add a guardian**: the person's device reads the offer and, with the kit
   still in the panel or the saved one, calls
   `replaceRecovery(session, [share], {threshold: 2, guardians: [{paper: true}, device]})`.
   It publishes `{kind: "prism/guardian-share/1", share}` and keeps the change,
   and its new kit (the paper share of two), only after the guardian answers
   `{kind: "prism/guardian-kept/1", recovery}`. The old kit is retired.
3. **Recovery with a guarded kit**: the new device publishes
   `{kind: "prism/guardian-help/1", prism, recovery, device}` and shows a
   `prism-help:` link and `recoveryCode(prism, device)`. **Help someone
   recover** shows the guardian the same code; the guardian releases only
   after hearing it from the person by voice or in person: `releaseShare`
   re-wraps the share to the new device, `{kind: "prism/guardian-release/1", share}`,
   and the guardian's device deletes its share. `recoverPrism` then combines
   the two shares, and the new kit is one paper share again, to which a
   guardian can be added.

A copy of a guarded kit recovers nothing alone, and the guardian who released
once has no share left for it: the guardian is the witness of the kit's use
that a paper kit lacks.

## Devices

The panel lists the id's active devices from its log and removes one with
`revokeDevice`, which rotates the private boundary key. It then shares the
log under every key it holds, so devices that have not taken up the rotation
still hear it.

## Apps hear removals

Every time a device shares (after creating, linking, removing or
recovering), it also publishes the id's public log with `publishLog`, on a
topic derived from the prism id alone. `verifySession` passes the relay to
`verifyAction` (`{bus}`), which merges what that topic holds (`heardLog`), so
a session from a device removed elsewhere is refused even when that device
never synced. Making the session and recovery check the same merged log.

## An app's workspace key

`frame.workspaceKey()` answers 32 bytes an app seals its own records with. The
frame makes them at random on first use, seals them with `sealPrivate` and
keeps `{prism, sealed}` in this origin's `prism-id` store under `workspace`;
the app never stores the key. A later call opens it with `openPrivate`, and
reseals it when the id's boundary key has rotated. Joining an id or recovering
one carries the key from the id this browser had (`carryWorkspace`), in the
same write that keeps the device. A key kept by another id is refused as
`removed`: this device was removed and made a new id. `{replace: true}` makes a
new key, for an app that starts again.

## An app on every device of the id

`frame.appChannel(listen)` resolves to `{send, stop}`. The topic is
`prism-app:` and the HMAC, under the current boundary key's topic key, of
`prism/app-channel/1 <origin>` (`appTopic`): only the id's devices derive it,
one per app, and a rotation moves it. `send` seals with `sealPrivate`;
`listen` gets only what `openPrivate` opens.

## Not yet (stitch's to define, its docs/prism-id.md "Open")

- Whoever holds a used paper kit (no guardian) can still sign a second
  retirement and publish it. Where the recovery that used the kit is heard,
  apps act for neither side; where it is not, the replay passes for a
  recovery. A guardian closes it (see Guardians); a kit that never left the
  panel before a guardian was added was never exposed.
- The relay holds messages for 30 minutes, so an app hears a removal only
  within 30 minutes of the last publish. A session signed before a removal
  lasts until it expires (at most 8 days).
