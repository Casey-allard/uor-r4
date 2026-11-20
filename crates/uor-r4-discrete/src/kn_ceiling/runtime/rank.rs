// The scalar lookup is the training oracle. For decoding we traverse each
// matching sorted follower list once for totals and once for all targets.
struct ActiveOrder<'a> {
    table: &'a CompactOrder<'a>,
    remaining: &'a [u8],
    adjusted: NgramContext,
}

impl CompactCountLookup<'_> {
    /// Emit all legal original target IDs in ascending order with their fit
    /// bucket and exact Q48 mass. A caller selects the lowest ID on a tie.
    /// The count and Pick operators use only the supplied causal prefix.
    pub fn visit_next_masses(
        &self,
        prefix: &[u16],
        fit: &FrozenPickMixture,
        picked: Option<(u16, u32)>,
        mut visit: impl FnMut(u16, u16, u64),
    ) -> Result<()> {
        if picked.is_some_and(|(id, _)| id == NGRAM_BOS || u32::from(id) >= NGRAM_VOCABULARY) {
            return Err(invalid("invalid A1 successor"));
        }
        let class = match picked {
            None => PICK_NOT_FIRED,
            Some((_, 0)) => PICK_FIRED_EXACT,
            Some(_) => PICK_FIRED_NEAR,
        };
        const WIDTH: usize = NGRAM_MAX_ORDER as usize + 1;
        let mut active: [Option<ActiveOrder<'_>>; WIDTH] = std::array::from_fn(|_| None);
        let mut totals = [0; WIDTH];
        for (k, table) in self
            .orders
            .iter()
            .enumerate()
            .take(prefix.len().saturating_add(1))
        {
            let key = PrimeRouteKey::from_context(&prefix[prefix.len() - k..])?;
            let body = key_body(key.word(), k as u32);
            let Some(followers) = table.matched_rows(&body[16 - table.key_width..]) else {
                continue;
            };
            let mut context = NgramContext {
                key: key.word(),
                total: 0,
                followers: 0,
                n1: 0,
                n2: 0,
                n3_plus: 0,
            };
            for row in followers.chunks_exact(follower_width(table.shared)) {
                let (_, raw, adjusted) = table.read_follower(row);
                totals[k] += u64::from(raw);
                context.total += u64::from(adjusted);
                context.followers += 1;
                match adjusted {
                    1 => context.n1 += 1,
                    2 => context.n2 += 1,
                    _ => context.n3_plus += 1,
                }
            }
            active[k] = Some(ActiveOrder {
                table,
                remaining: followers,
                adjusted: context,
            });
        }
        let width = self.orders.len();
        for id in 1..NGRAM_VOCABULARY as u16 {
            let mut counts = [0; WIDTH];
            let mut mass = u128::from(KN_BASE_MASS);
            for (order, slot) in active[..width].iter_mut().enumerate() {
                let Some(level) = slot else { continue };
                let mut adjusted_target = 0;
                let stride = follower_width(level.table.shared);
                if let Some(row) = level.remaining.get(..stride) {
                    let (candidate, raw, adjusted) = level.table.read_follower(row);
                    if candidate == id {
                        counts[order] = raw;
                        adjusted_target = adjusted;
                        level.remaining = &level.remaining[stride..];
                    }
                }
                mass =
                    interpolate_order(mass, adjusted_target, level.adjusted, level.table.discounts);
            }
            let pick = PickTarget {
                class,
                hit: picked.is_some_and(|(token, _)| token == id),
            };
            let kn =
                u64::try_from(mass.max(1)).map_err(|_| invalid("compact KN mass exceeded u64"))?;
            let (bucket, q48) = fit.predict_mass(kn, &counts[..width], &totals[..width], pick)?;
            visit(id, bucket, q48);
        }
        Ok(())
    }
}
