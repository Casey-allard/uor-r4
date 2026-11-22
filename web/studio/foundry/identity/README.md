# Prism ids

There is no sign-in. The first time a visitor touches a Prism app, the frame
makes them a Prism id in the background: no screen, no prompt, no password,
no passkey and no account server. The browser makes its own key, which never
leaves it. The person icon at the right end of the bar shows the id and its
quiet options; one Prism id serves every Prism app, Foundry included.
[PROTOCOL.md](PROTOCOL.md) is the contract. The id itself is stitch's Prism
id, pinned in [stitch.lock](stitch.lock), and nothing here is deployed beyond
the frame.

## One id, every app and device

Each app keeps its own device of the id, in its own origin's storage. To use
the same id in another app, or on a phone:

1. In an app that has your id, open the person icon and choose **Add a device or app**. Scan the QR code with a phone, or copy the link.
2. In the new app, choose **Use my Prism id from another device or app** and paste it.
3. The new app shows six digits. Type them into the first app.

This is stitch's link offer. The link holds a one-time secret, good for five
minutes. The six digits bind that secret to the device that answered, so
someone who copied the link first gets different digits. The two devices meet
on stitch's relay, under a name derived from that secret.

## Losing every device never loses the id

A new id comes with a **recovery kit**: a file holding the paper share of the
id's recovery device (stitch's threshold recovery, one share of one) and the
id's log. It waits in the person icon's panel (**Save recovery kit**) until it
is saved; nothing asks for it. On any browser, **Use a recovery kit** brings the id back.
That browser becomes the id's device, every other device is removed, and a
new kit replaces the old one.

Recovery restores the id: the same id, everywhere. It does not
restore what an app kept only in a lost browser (its local projects, chats or
databases). That lives with the app, not the id.

Stitch lets only the recovery device replace itself, so a member cannot make
a new kit. A kit is made when the id is created, again on each recovery, and
when a guardian is added.

**Add a guardian** (optional, nothing asks for it) gives someone you trust,
in any Prism app (**Be someone's guardian**), one share of the kit: the new
kit then needs them too. If you lose every device, they open **Help someone
recover**, hear the code your new browser shows from you, and release their
share once. A lost or copied kit alone then brings nothing back. Add the
guardian before saving the first kit, and no kit that works alone ever
leaves the browser.

## Your work, sealed to your id

An app can keep a visitor's work sealed to their id: the frame hands it a
key that only the id's devices open, and carries it when this app joins the
id from another device or recovers it. A device removed from the id cannot
open the work it kept. See PROTOCOL.md, "An app's workspace key".

## Devices

**Devices** lists the id's devices. **Remove** is stitch's `revokeDevice`,
which rotates the id's private key; the id itself never changes. A removed
device stops signing the next time it syncs. The recovery device has no
Remove, because stitch lets no member revoke it.

## Who is trusted with what

| Part | If it were hostile, it could | It could not |
| --- | --- | --- |
| stitch's relay (ntfy) | Withhold or delay a sync message or a link answer, so a removal or a link does not arrive | Read a sync message (sealed in the id's private boundary) or forge any record (each is signed and checked by the reader). It sees hashed topic names, timing, padded sizes, and in a link answer the id's log (public keys and signatures) |
| The app's page (its script) | Everything this origin's device can do: sign for this app, link or remove devices, read a recovery kit as it is made | Copy the device keys out: they are non-extractable WebCrypto keys |
| Another app | Make its own device of the visitor's id | Use this app's device or session: storage is per origin, and a session names the app origin it was signed for |
| Anyone holding a recovery kit | Take the id over, from any browser, if the kit has no guardian | Use a kit with a guardian without the guardian releasing to them; use a kit once a newer kit replaced it, if they hear what the id's devices shared (see below) |
| A guardian | Refuse to help, or help someone else holding the kit | Recover the id alone: their share is one of two, and the kit holds the other |

There is no biometric or OS prompt: a device's keys sign whenever its page
asks, and the visitor's click is the only consent.

A session lasts at most 8 days (the frame asks for 7). A removed device stops
signing new sessions once it syncs, but a session it signed earlier stays
valid until it expires. Every device also publishes the id's public log on
the relay, and an app checking a session merges it, so a stolen browser that
never syncs is refused once another device removed it and shared that.

The relay keeps a message for stitch's warm lifetime (30 minutes). A device
away longer, or an app checking later, catches up from the next message a
device shares. A used kit replayed against its own old log binds and revokes
nothing: the retirement in the id's log never kept what the replay signs, so
the frame refuses the kit once that retirement shows up in what the devices
shared, and publishes nothing.

## Run it

```sh
npm ci && npx playwright install chromium
npm run test:identity        # the stitch copy matches its lock, every refused check, then the flow in Chromium
node identity/dev.mjs        # two demo apps, on 127.0.0.1:8791 and :8792
```

`test.mjs` puts an in-memory relay in place of the real one in the browser.
`dev.mjs` run by hand uses the real relay.

## Follow stitch

```sh
node identity/stitch-sync.mjs <stitch commit>    # fetches that commit; add a local checkout path to read it instead
```

This rewrites `prism-id.mjs` and `stitch.lock`. Copy `frame/` and `identity/`
to Foundry and prism-devrel byte for byte, as with any frame change.
