//! Generates the JSON search index and search.js client.

use anyhow::Result;

use crate::extractor::build_search_index;

/// Generates the JSON search index as a serialized string.
///
/// # Errors
///
/// Returns an error if JSON serialization fails.
pub fn generate_search_index(base_path: &str) -> Result<String> {
    let entries = build_search_index(base_path);
    let json = serde_json::to_string(&entries)
        .map_err(|e| anyhow::anyhow!("Failed to serialize search index: {}", e))?;
    Ok(json)
}

/// Returns the client-side search JavaScript with scoring, facets, and kind badges.
pub fn search_js(base_path: &str) -> String {
    format!(
        r"// UOR Foundation — client-side search
// Reads search-index.json and filters + ranks results on input

(function () {{
  'use strict';

  var input    = document.getElementById('search-input');
  var results  = document.getElementById('search-results');
  if (!input || !results) return;

  var index = [];
  var activeSpaces = new Set();
  var activeKinds  = new Set();

  fetch('{base_path}/search-index.json')
    .then(function (r) {{ return r.json(); }})
    .then(function (data) {{ index = data; }})
    .catch(function (e) {{ console.error('Failed to load search index:', e); }});

  // Scoring: exact label match > prefix match > label contains > description contains
  function score(entry, q) {{
    var l = entry.label.toLowerCase();
    if (l === q)             return 100;
    if (l.startsWith(q))     return 80;
    if (l.includes(q))       return 60;
    if ((entry.description||'').toLowerCase().includes(q)) return 20;
    return 0;
  }}

  function createBadge(cls, text) {{
    var span = document.createElement('span');
    span.className = 'badge ' + cls;
    span.textContent = text;
    return span;
  }}

  function render(matches) {{
    results.innerHTML = '';
    if (matches.length === 0) {{
      var li = document.createElement('li');
      li.textContent = 'No results.';
      results.appendChild(li);
      return;
    }}
    matches.forEach(function (entry) {{
      var li  = document.createElement('li');
      li.className = 'animate-in';
      var a   = document.createElement('a');
      a.href  = entry.url;
      a.textContent = entry.label;
      li.appendChild(a);
      // Kind badge
      if (entry.kind) {{
        li.appendChild(createBadge('badge-' + entry.kind, entry.kind));
      }}
      // Space badge
      if (entry.space) {{
        li.appendChild(createBadge('badge-' + entry.space, entry.space));
      }}
      // Sub-kind badge
      if (entry.subkind) {{
        li.appendChild(createBadge('badge-' + entry.subkind, entry.subkind));
      }}
      var desc = document.createElement('p');
      desc.className = 'result-desc';
      desc.textContent = entry.description || '';
      li.appendChild(desc);
      results.appendChild(li);
    }});
  }}

  function search() {{
    var q = (input.value || '').trim().toLowerCase();
    results.innerHTML = '';
    if (q.length < 2) return;

    var scored = index
      .map(function (e) {{ return {{ entry: e, s: score(e, q) }}; }})
      .filter(function (x) {{ return x.s > 0; }})
      .filter(function (x) {{
        if (activeSpaces.size > 0 && !activeSpaces.has(x.entry.space)) return false;
        if (activeKinds.size  > 0 && !activeKinds.has(x.entry.kind))  return false;
        return true;
      }})
      .sort(function (a, b) {{ return b.s - a.s; }})
      .slice(0, 30)
      .map(function (x) {{ return x.entry; }});

    render(scored);
  }}

  input.addEventListener('input', search);

  // Facet checkbox listeners
  document.querySelectorAll('.facet-space').forEach(function (cb) {{
    cb.addEventListener('change', function () {{
      if (cb.checked) activeSpaces.add(cb.value);
      else activeSpaces.delete(cb.value);
      search();
    }});
  }});

  document.querySelectorAll('.facet-kind').forEach(function (cb) {{
    cb.addEventListener('change', function () {{
      if (cb.checked) activeKinds.add(cb.value);
      else activeKinds.delete(cb.value);
      search();
    }});
  }});

  // Restore query from URL ?q= parameter
  var params = new URLSearchParams(window.location.search);
  var qParam = params.get('q');
  if (qParam) {{
    input.value = qParam;
    search();
  }}
}}());

// Multi-level dropdown support — Bootstrap 5 does not handle nested dropdowns
// natively. This toggles .show on nested .dropend submenus when clicked on
// mobile (collapsed navbar) and prevents the parent dropdown from closing.
(function () {{
  'use strict';
  document.querySelectorAll('.site-nav .dropend > .dropdown-toggle').forEach(function (toggle) {{
    toggle.addEventListener('click', function (e) {{
      var submenu = toggle.nextElementSibling;
      if (!submenu || !submenu.classList.contains('dropdown-menu')) return;
      e.preventDefault();
      e.stopPropagation();
      // Close sibling submenus
      var parent = toggle.closest('.dropdown-menu');
      if (parent) {{
        parent.querySelectorAll(':scope > .dropend > .dropdown-menu.show').forEach(function (m) {{
          if (m !== submenu) m.classList.remove('show');
        }});
      }}
      submenu.classList.toggle('show');
    }});
  }});
  // Close nested submenus when the parent dropdown closes
  document.querySelectorAll('.site-nav .nav-item.dropdown').forEach(function (dropdown) {{
    dropdown.addEventListener('hidden.bs.dropdown', function () {{
      dropdown.querySelectorAll('.dropdown-menu.show').forEach(function (m) {{
        m.classList.remove('show');
      }});
    }});
  }});
}}());
",
        base_path = base_path,
    )
}
