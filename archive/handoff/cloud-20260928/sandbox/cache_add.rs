    add(
        &mut builder,
        "head",
        &head,
        s.vocab,
        s.width,
        Some(Site::Head),
    )?;
    if let Some(cache) = cache {
        if cache.geometry != "lorentz" || cache.dim == 0 || cache.gap == 0 {
            return Err(invalid("only a Lorentz cache is served"));
        }
        if cache.query.len() != s.width * cache.dim || cache.gate_weight.len() != s.width {
            return Err(invalid("cache maps do not match the backbone width"));
        }
        // The maps act on the final normalized state: rows are the dim outputs.
        let transpose = |map: &[f32]| -> Vec<f32> {
            (0..cache.dim)
                .flat_map(|d| (0..s.width).map(move |w| map[w * cache.dim + d]))
                .collect()
        };
        for (name, map) in [("cache.query", &cache.query), ("cache.key", &cache.key)] {
            add(&mut builder, name, &transpose(map), cache.dim, s.width, Some(Site::Head))?;
        }
        add(&mut builder, "cache.gate", &cache.gate_weight, 1, s.width, Some(Site::Head))?;
        let (beta_m, beta_e) = grid_nearest(f64::from(cache.log_beta).exp());
        builder.set_cache(CacheSpec {
            geometry: "lorentz".to_owned(),
            dim: cache.dim,
            gap: cache.gap,
            beta_m,
            beta_e,
            gate_bias: (f64::from(cache.gate_bias) * 65536.0).round() as i64,
        });
        builder
            .add_table("arcosh", TableValues::U32(&arcosh_table()))
            .map_err(|e| invalid(e.to_string()))?;
    }
