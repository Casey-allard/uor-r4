"""Registry readers: skills, roles, mistakes, milestones, panels.

Everything the harness knows about the project's own contract is read here, once,
into plain dataclasses. Other modules depend on this module and never re-parse.
"""

from __future__ import annotations

import fnmatch
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable

from core import HarnessPaths, load_json, parse_front_matter

# --------------------------------------------------------------------------- skills


@dataclass
class Skill:
    id: str
    name: str
    description: str
    path: Path
    body: str
    version: str = ""
    harness: str = ""
    kind: str = "skill"
    appliesto: list[str] = field(default_factory=list)
    when: list[str] = field(default_factory=list)
    requires: list[str] = field(default_factory=list)
    outputs: list[str] = field(default_factory=list)
    forbidden: list[str] = field(default_factory=list)
    milestones: list[str] = field(default_factory=list)

    @property
    def trigger(self) -> str:
        return " ".join(self.when) if self.when else self.description


def load_skills(paths: HarnessPaths) -> list[Skill]:
    if not paths.skills_dir.is_dir():
        return []
    out: list[Skill] = []
    for skill_file in sorted(paths.skills_dir.glob("*/SKILL.md")):
        fm, body = parse_front_matter(skill_file.read_text(encoding="utf-8"))
        if not fm.get("name"):
            raise ValueError(f"{skill_file}: front matter must set 'name'")
        if not fm.get("description"):
            raise ValueError(f"{skill_file}: front matter must set 'description'")
        out.append(
            Skill(
                id=skill_file.parent.name,
                name=str(fm.get("name")),
                description=str(fm.get("description")),
                path=skill_file,
                body=body.strip(),
                version=str(fm.get("version", "")),
                harness=str(fm.get("harness", "")),
                kind=str(fm.get("kind", "skill")),
                appliesto=_as_list(fm.get("applies-to")),
                when=_as_list(fm.get("when")),
                requires=_as_list(fm.get("requires")),
                outputs=_as_list(fm.get("outputs")),
                forbidden=_as_list(fm.get("forbidden")),
                milestones=_as_list(fm.get("milestones")),
            )
        )
    return out


def _as_list(value: Any) -> list[str]:
    if value is None or value == "":
        return []
    if isinstance(value, list):
        return [str(v) for v in value]
    return [str(value)]


def find_skill(skills: Iterable[Skill], key: str) -> Skill | None:
    for s in skills:
        if s.id == key or s.name == key:
            return s
    for s in skills:
        if s.id.startswith(key):
            return s
    return None


def match_skills(skills: Iterable[Skill], query: str) -> list[tuple[Skill, str]]:
    """Rank skills against a free-text task description or a list of paths.

    A `when` trigger is the skill author saying "use me for this". It ranks far above
    an incidental description-word overlap, otherwise a generic word like "run" pulls
    in every skill and the routing becomes noise.
    """
    q = query.strip().lower()
    tokens = {t for t in re.split(r"[^a-z0-9_.\-/]+", q) if len(t) > 2}
    strong: list[tuple[int, Skill, str]] = []
    weak: list[tuple[int, Skill, str]] = []
    for s in skills:
        when_text = " ".join(s.when).lower()
        hay_desc = f"{s.name} {s.description} {s.id}".lower()
        hits: list[str] = []
        weight = 0
        for trigger in s.when:
            t = trigger.lower().strip()
            if not t:
                continue
            if q and t and t in q:
                hits.append(f"when:{t}")
                weight += 60 + len(t.split())
        for t in sorted(tokens):
            if t in when_text:
                hits.append(f"when~{t}")
                weight += 10
            elif t in hay_desc:
                hits.append(f"desc~{t}")
                weight += 1
            else:
                for pat in s.appliesto:
                    if fnmatch.fnmatch(t, pat.lower()) or fnmatch.fnmatch(pat.lower(), f"*{t}*"):
                        hits.append(f"path~{pat}")
                        weight += 8
                        break
        if not hits:
            continue
        entry = (weight, s, ", ".join(sorted(set(hits))[:4]))
        (strong if weight >= 10 else weak).append(entry)
    strong.sort(key=lambda e: (-e[0], e[1].id))
    weak.sort(key=lambda e: (-e[0], e[1].id))
    # only fall back to incidental matches when nothing triggered
    ranked = strong if strong else weak
    return [(s, why) for _, s, why in ranked]


# ---------------------------------------------------------------------------- roles


@dataclass
class Role:
    id: str
    title: str
    seat: str
    provider: str
    model: str
    reasoning: str
    purpose: str
    skills: list[str] = field(default_factory=list)
    when: list[str] = field(default_factory=list)
    read_only: bool = False

    @property
    def route(self) -> str:
        return f"{self.provider}/{self.model}" if self.provider else "(inherit)"


def load_roles(paths: HarnessPaths) -> list[Role]:
    if not paths.roles_json.exists():
        return []
    data = load_json(paths.roles_json)
    out: list[Role] = []
    for rid, raw in (data.get("roles") or {}).items():
        route = raw.get("route") or {}
        out.append(
            Role(
                id=rid,
                title=str(raw.get("title", rid)),
                seat=str(raw.get("seat", "specialist")),
                provider=str(route.get("provider", "")),
                model=str(route.get("model", "")),
                reasoning=str(route.get("reasoning", "")),
                purpose=str(raw.get("purpose", "")),
                skills=_as_list(raw.get("skills")),
                when=_as_list(raw.get("when")),
                read_only=bool(raw.get("read_only", False)),
            )
        )
    return out


def find_role(roles: Iterable[Role], key: str) -> Role | None:
    for r in roles:
        if r.id == key:
            return r
    for r in roles:
        if r.id.startswith(key):
            return r
    return None


# ------------------------------------------------------------------------- mistakes

SEVERITIES = ("block", "warn", "info")


@dataclass
class Mistake:
    id: str
    title: str
    path: Path
    body: str
    date: str = ""
    severity: str = "warn"
    class_: str = ""
    trigger: list[str] = field(default_factory=list)
    paths: list[str] = field(default_factory=list)
    milestones: list[str] = field(default_factory=list)
    mechanism: str = ""
    guard: str = ""
    evidence: list[str] = field(default_factory=list)
    status: str = "active"

    @property
    def lesson(self) -> str:
        m = re.search(r"^##\s+Lesson\s*\n+(.+?)(?=\n##\s|\Z)", self.body, re.DOTALL | re.MULTILINE)
        return _first_paragraph(m.group(1)) if m else ""

    @property
    def symptom(self) -> str:
        m = re.search(r"^##\s+Symptom\s*\n+(.+?)(?=\n##\s|\Z)", self.body, re.DOTALL | re.MULTILINE)
        return _first_paragraph(m.group(1)) if m else ""


def _first_paragraph(text: str) -> str:
    for para in re.split(r"\n\s*\n", text.strip()):
        para = " ".join(para.split())
        if para:
            return para
    return ""


def load_mistakes(paths: HarnessPaths) -> list[Mistake]:
    if not paths.mistakes_dir.is_dir():
        return []
    out: list[Mistake] = []
    for f in sorted(paths.mistakes_dir.glob("*.md")):
        if f.name.upper() == "README.MD":
            continue
        fm, body = parse_front_matter(f.read_text(encoding="utf-8"))
        if not fm.get("id"):
            raise ValueError(f"{f}: front matter must set 'id'")
        sev = str(fm.get("severity", "warn"))
        if sev not in SEVERITIES:
            raise ValueError(f"{f}: severity must be one of {SEVERITIES}, got {sev!r}")
        out.append(
            Mistake(
                id=str(fm["id"]),
                title=str(fm.get("title", fm["id"])),
                path=f,
                body=body.strip(),
                date=str(fm.get("date", "")),
                severity=sev,
                class_=str(fm.get("class", "")),
                trigger=_as_list(fm.get("trigger")),
                paths=_as_list(fm.get("paths")),
                milestones=_as_list(fm.get("milestones")),
                mechanism=str(fm.get("mechanism", "")),
                guard=str(fm.get("guard", "")),
                evidence=_as_list(fm.get("evidence")),
                status=str(fm.get("status", "active")),
            )
        )
    return out


def find_mistake(mistakes: Iterable[Mistake], key: str) -> Mistake | None:
    for m in mistakes:
        if m.id == key:
            return m
    for m in mistakes:
        if m.id.startswith(key):
            return m
    return None


def match_mistakes(mistakes: Iterable[Mistake], files: Iterable[str] = (), query: str = "", milestones: Iterable[str] = ()) -> list[tuple[Mistake, str]]:
    """Select lessons that apply to a set of files, a query, or a milestone."""
    files = [str(f) for f in files]
    ms = {str(m) for m in milestones}
    q = query.lower().strip()
    q_tokens = {t for t in re.split(r"[^a-z0-9_\-]+", q) if len(t) > 3}
    out: list[tuple[Mistake, str]] = []
    for m in mistakes:
        if m.status not in ("active", "warn"):
            continue
        why: list[str] = []
        for f in files:
            for pat in m.paths:
                if fnmatch.fnmatch(f, pat) or fnmatch.fnmatch(f, f"*{pat}*"):
                    why.append(f"path {f}")
                    break
        for pat in m.trigger:
            if q and (pat.lower() in q or any(t in pat.lower() for t in q_tokens)):
                why.append(f"trigger {pat}")
        if ms and set(m.milestones) & ms:
            why.append("same milestone")
        if q and m.mechanism and m.mechanism.lower() in q:
            why.append(f"mechanism {m.mechanism}")
        if why:
            out.append((m, "; ".join(sorted(set(why))[:3])))
    severity_rank = {"block": 0, "warn": 1, "info": 2}
    out.sort(key=lambda pair: (severity_rank.get(pair[0].severity, 3), pair[0].id))
    return out


# ----------------------------------------------------------------------- milestones


@dataclass
class Milestone:
    key: str
    issue: int
    id: str
    title: str
    lab: str
    status: str
    headline: dict[str, Any]
    secondary: list[dict[str, Any]]
    panel: str
    base_artifact: str
    mechanisms: list[str]
    lines: list[dict[str, Any]]
    next: str

    @property
    def slug(self) -> str:
        return f"{self.id} #{self.issue}"

    def headline_line(self) -> str:
        h = self.headline
        if h.get("vacuous"):
            return f"{h['label']}: not qualified"
        if h.get("text"):
            # some live headlines are qualitative ("softmax at runtime: yes") and only
            # become a number once the milestone is satisfied
            return f"{self.id}: {h['text']} -> target {h.get('target_display', h.get('target'))}"
        cur = _fmt_num(h.get("current"))
        tgt = _fmt_num(h.get("target"))
        of = h.get("of")
        suffix = f"/{of}" if of else ""
        return f"{self.id}: {h['label']} {cur}{suffix} -> target {tgt}{suffix}"

    def line_counts(self) -> str:
        return ", ".join(f"{l['name']} {l['count']}/{l['of']}" for l in self.lines)


def _fmt_num(v: Any) -> str:
    if isinstance(v, float):
        return f"{v:g}"
    return str(v)


@dataclass
class Milestones:
    tracker: dict[str, Any]
    owner_rules: list[dict[str, str]]
    by_key: dict[str, Milestone]

    def get(self, key: str) -> Milestone | None:
        k = key.strip().lower()
        if k in self.by_key:
            return self.by_key[k]
        for m in self.by_key.values():
            if k in (m.id.lower(), str(m.issue), f"#{m.issue}"):
                return m
        return None

    def by_lab(self, lab: str) -> list[Milestone]:
        return [m for m in self.by_key.values() if m.lab == lab.lower()]

    def all(self) -> list[Milestone]:
        return sorted(self.by_key.values(), key=lambda m: m.issue)


def load_milestones(paths: HarnessPaths) -> Milestones:
    data = load_json(paths.milestones_json)
    by_key: dict[str, Milestone] = {}
    for key, raw in (data.get("milestones") or {}).items():
        by_key[key] = Milestone(
            key=key,
            issue=int(raw["issue"]),
            id=str(raw["id"]),
            title=str(raw["title"]),
            lab=str(raw.get("lab", "")),
            status=str(raw.get("status", "")),
            headline=dict(raw.get("headline") or {}),
            secondary=list(raw.get("secondary") or []),
            panel=str(raw.get("panel", "")),
            base_artifact=str(raw.get("base_artifact", "")),
            mechanisms=_as_list(raw.get("mechanisms")),
            lines=list(raw.get("lines") or []),
            next=str(raw.get("next", "")),
        )
    return Milestones(tracker=dict(data.get("tracker") or {}), owner_rules=list(data.get("owner_rules") or []), by_key=by_key)


# --------------------------------------------------------------------------- panels


@dataclass
class Panel:
    id: str
    title: str
    milestone: str
    frozen: bool
    grader: str
    bars: list[dict[str, Any]]
    noise: str = ""
    seeds: str = ""
    rows: str = ""
    raw: dict[str, Any] = field(default_factory=dict)


def load_panels(paths: HarnessPaths) -> list[Panel]:
    if not paths.panels_dir.is_dir():
        return []
    out: list[Panel] = []
    for f in sorted(paths.panels_dir.glob("*.json")):
        data = load_json(f)
        for pid, raw in (data.get("panels") or {}).items():
            out.append(
                Panel(
                    id=pid,
                    title=str(raw.get("title", pid)),
                    milestone=str(raw.get("milestone", "")),
                    frozen=bool(raw.get("frozen", False)),
                    grader=str(raw.get("grader", "")),
                    bars=list(raw.get("bars") or []),
                    noise=str(raw.get("noise", "")),
                    seeds=str(raw.get("seeds", "")),
                    rows=str(raw.get("rows", "")),
                    raw=dict(raw),
                )
            )
    return out


def find_panel(panels: Iterable[Panel], key: str) -> Panel | None:
    for p in panels:
        if p.id == key:
            return p
    for p in panels:
        if key in p.id or p.id in key:
            return p
    return None


def panel_for_milestone(panels: Iterable[Panel], milestone: Milestone) -> Panel | None:
    return find_panel(panels, milestone.panel)


# ------------------------------------------------------------------------- decisions


@dataclass
class Decision:
    id: str
    title: str
    line: int
    body: str
    date: str = ""


_DEC_RE = re.compile(r"^##\s+(D\d+[a-z]?)\s*(?:—|-|:)\s*(.+)$", re.MULTILINE)


def load_decisions(paths: HarnessPaths) -> list[Decision]:
    """Parse DECISIONS.md headings. The file stays authoritative; this is a view."""
    if not paths.decisions_md.exists():
        return []
    text = paths.decisions_md.read_text(encoding="utf-8", errors="replace")
    matches = list(_DEC_RE.finditer(text))
    out: list[Decision] = []
    for i, m in enumerate(matches):
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        body = text[m.end() : end]
        date = ""
        dm = re.search(r"\b(\d{1,2}\s+\w+\s+20\d\d)\b", body[:400])
        if dm:
            date = dm.group(1)
        out.append(Decision(id=m.group(1), title=m.group(2).strip(), line=text[: m.start()].count("\n") + 1, body=body.strip(), date=date))
    return out


def find_decision(decisions: Iterable[Decision], key: str) -> Decision | None:
    k = key.strip().upper()
    for d in decisions:
        if d.id.upper() == k:
            return d
    return None


# ----------------------------------------------------------------------- repo cache


@dataclass
class Registry:
    paths: HarnessPaths
    skills: list[Skill]
    roles: list[Role]
    mistakes: list[Mistake]
    milestones: Milestones
    panels: list[Panel]
    decisions: list[Decision]

    @classmethod
    def load(cls, paths: HarnessPaths) -> "Registry":
        return cls(
            paths=paths,
            skills=load_skills(paths),
            roles=load_roles(paths),
            mistakes=load_mistakes(paths),
            milestones=load_milestones(paths),
            panels=load_panels(paths),
            decisions=load_decisions(paths),
        )

    def milestone_or_fail(self, key: str) -> Milestone:
        m = self.milestones.get(key)
        if m is None:
            raise KeyError(f"unknown milestone {key!r}; known: {', '.join(sorted(self.milestones.by_key))}")
        return m
