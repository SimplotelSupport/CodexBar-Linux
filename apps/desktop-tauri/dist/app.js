const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

const $ = (sel) => document.querySelector(sel);
const main = $("#main");
const empty = $("#empty");
const statusEl = $("#status");
const tpl = document.getElementById("row-template");

function pctClass(pct) {
  if (pct >= 90) return "crit";
  if (pct >= 70) return "warn";
  return "";
}

function makeBar({ label, pct, reset }) {
  const wrap = document.createElement("div");
  wrap.className = "bar";

  const lbl = document.createElement("span");
  lbl.className = "bar-label";
  lbl.textContent = label;

  const track = document.createElement("div");
  track.className = "bar-track";
  const fill = document.createElement("div");
  const safePct = Math.max(0, Math.min(100, Number(pct) || 0));
  fill.className = "bar-fill " + pctClass(safePct);
  fill.style.width = `${safePct}%`;
  track.appendChild(fill);

  const num = document.createElement("span");
  num.className = "bar-pct";
  num.textContent = `${Math.round(safePct)}%`;

  wrap.appendChild(lbl);
  wrap.appendChild(track);
  wrap.appendChild(num);

  if (reset) {
    const r = document.createElement("span");
    r.className = "bar-reset";
    r.textContent = `resets in ${reset}`;
    wrap.appendChild(r);
  }
  return wrap;
}

function renderProvider(p) {
  const node = tpl.content.firstElementChild.cloneNode(true);
  node.querySelector(".pname").textContent = p.display_name;
  node.querySelector(".psource").textContent = p.source || "";

  const accountEl = node.querySelector(".account");
  const accountParts = [];
  if (p.account_email) accountParts.push(p.account_email);
  if (p.login_method) accountParts.push(p.login_method);
  accountEl.textContent = accountParts.join(" · ");

  const bars = node.querySelector(".bars");
  if (!p.error) {
    bars.appendChild(makeBar({
      label: p.primary_label || "Session",
      pct: p.primary_pct,
      reset: p.primary_reset
    }));
    if (p.secondary_label) {
      bars.appendChild(makeBar({
        label: p.secondary_label,
        pct: p.secondary_pct || 0,
        reset: p.secondary_reset
      }));
    }
  }

  const cost = node.querySelector(".cost");
  if (p.cost_used) {
    const limit = p.cost_limit ? ` / ${p.cost_limit}` : "";
    const period = p.cost_period ? ` · ${p.cost_period}` : "";
    cost.textContent = `Cost: ${p.cost_used}${limit}${period}`;
  }

  const err = node.querySelector(".error");
  if (p.error) {
    err.textContent = p.error;
  }
  return node;
}

function render(snap) {
  if (!snap || !snap.providers) {
    return;
  }
  main.innerHTML = "";
  if (snap.providers.length === 0) {
    main.appendChild(empty);
    empty.textContent = "No providers configured yet.";
    return;
  }
  for (const p of snap.providers) {
    main.appendChild(renderProvider(p));
  }
  const ts = new Date(Number(snap.fetched_at) * 1000);
  statusEl.textContent = `Updated ${ts.toLocaleTimeString()}`;
}

async function refresh() {
  statusEl.textContent = "Refreshing…";
  try {
    const snap = await invoke("refresh_usage");
    render(snap);
  } catch (err) {
    statusEl.textContent = `Error: ${err}`;
    console.error(err);
  }
}

async function init() {
  const cached = await invoke("get_cached_snapshot");
  if (cached) render(cached);

  await listen("usage-updated", (event) => render(event.payload));

  document.getElementById("refresh").addEventListener("click", refresh);
  document.getElementById("hide").addEventListener("click", async () => {
    await getCurrentWindow().hide();
  });

  if (!cached) {
    refresh();
  }
}

init();
