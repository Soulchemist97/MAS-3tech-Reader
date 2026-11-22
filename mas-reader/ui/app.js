"use strict";

// Ohne Bundler: die Tauri-API kommt über window.__TAURI__ (withGlobalTauri).
// Texte aus den Dateien werden nur per textContent gesetzt, nie als HTML.
const { invoke } = window.__TAURI__.core;
const { open, save } = window.__TAURI__.dialog;

const $ = (selector) => document.querySelector(selector);

const isMac = /Mac/.test(navigator.userAgent);
if (isMac) document.documentElement.classList.add("macos");

// ---------------------------------------------------------------------------
// Kleine Helfer
// ---------------------------------------------------------------------------

const store = {
  get(key, fallback) {
    try {
      const raw = localStorage.getItem("mas." + key);
      return raw === null ? fallback : JSON.parse(raw);
    } catch {
      return fallback;
    }
  },
  set(key, value) {
    localStorage.setItem("mas." + key, JSON.stringify(value));
  },
};

/** Element bauen. Strings als Kinder werden zu Textknoten (kein HTML). */
function el(tag, props = {}, ...children) {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(props)) {
    if (key === "class") node.className = value;
    else if (key.startsWith("on")) node.addEventListener(key.slice(2), value);
    else node.setAttribute(key, value);
  }
  node.append(...children);
  return node;
}

const plural = (n, one, many) => (n === 1 ? one : many);
const errorText = (e) => (typeof e === "string" ? e : (e && e.message) || String(e));

// ---------------------------------------------------------------------------
// Zustand
// ---------------------------------------------------------------------------

const S = {
  folder: "",
  location: "",
  columns: [],
  rows: [],              // { index, file_name, cells[] }
  sortCol: null,
  sortDir: 1,
  query: "",
  selected: new Set(),   // Zeilen-Indizes
  active: null,          // Zeile in der Schnellansicht
  anchor: null,          // Startzeile für Shift-Auswahl
  template: "",
  defaultTemplate: "",
  placeholders: [],
  texts: new Map(),      // Bon-Text je Index
  busy: false,
  outDir: "",            // Zielordner fürs Umbenennen
};

const sep = () => (S.folder.includes("\\") && !S.folder.includes("/") ? "\\" : "/");

// ---------------------------------------------------------------------------
// Rückmeldungen
// ---------------------------------------------------------------------------

let noticeTimer = null;

/** Unten eingeblendete Rückmeldung. `warnings`/`errors` erscheinen aufklappbar. */
function notify({ kind = "ok", title, text = "", warnings = [], errors = [] }) {
  clearTimeout(noticeTimer);
  const box = $("#notice");
  box.classList.toggle("error", kind === "error");
  $("#notice-title").textContent = title;
  $("#notice-text").textContent = text;

  const list = $("#notice-list");
  list.replaceChildren(
    ...warnings.map((w) => el("li", {}, w)),
    ...errors.map((e) => el("li", { class: "err" }, e)),
  );
  const hasDetails = warnings.length + errors.length > 0;
  $("#notice-details").hidden = !hasDetails;
  $("#notice-details").open = errors.length > 0;
  const parts = [];
  if (errors.length) parts.push(`${errors.length} ${plural(errors.length, "Fehler", "Fehler")}`);
  if (warnings.length) parts.push(`${warnings.length} ${plural(warnings.length, "Hinweis", "Hinweise")}`);
  $("#notice-summary").textContent = parts.join(", ");

  box.hidden = false;
  if (kind === "ok" && !hasDetails) noticeTimer = setTimeout(() => (box.hidden = true), 8000);
}

function notifyError(error, title = "Das hat nicht geklappt") {
  notify({ kind: "error", title, text: errorText(error) });
}

function setStatus(text) {
  $("#status").textContent = text;
}

// ---------------------------------------------------------------------------
// Tabelle
// ---------------------------------------------------------------------------

function sortKey(value, kind) {
  if (value === "") return null; // Fehlendes sortiert immer ans Ende
  if (kind === "date") {
    const [d, m, y] = value.split(".");
    return Number(y + m + d);
  }
  if (kind === "money") return parseFloat(value.replace(/\./g, "").replace(",", "."));
  return value;
}

function visibleRows() {
  const query = S.query.trim().toLowerCase();
  let rows = S.rows.filter((r) => !query || (r.file_name + " " + r.cells.join(" ")).toLowerCase().includes(query));

  if (S.sortCol !== null) {
    const kind = S.columns[S.sortCol].kind;
    const key = (r) => sortKey(r.cells[S.sortCol], kind);
    rows = [...rows].sort((a, b) => {
      const ka = key(a);
      const kb = key(b);
      if (ka === null && kb === null) return 0;
      if (ka === null) return 1;
      if (kb === null) return -1;
      const diff = typeof ka === "string" ? ka.localeCompare(kb, "de", { numeric: true }) : ka - kb;
      return diff * S.sortDir;
    });
  }
  return rows;
}

function sortBy(col) {
  if (S.sortCol === col) S.sortDir = -S.sortDir;
  else {
    S.sortCol = col;
    S.sortDir = 1;
  }
  renderTable();
}

function renderTable() {
  const head = el("tr");
  S.columns.forEach((column, i) => {
    const sorted = S.sortCol === i ? (S.sortDir === 1 ? "ascending" : "descending") : "none";
    head.append(
      el("th", { scope: "col", class: column.kind === "money" ? "num" : "", "aria-sort": sorted, onclick: () => sortBy(i) }, column.title),
    );
  });
  $("#table thead").replaceChildren(head);

  const rows = visibleRows().map((row) => {
    const tr = el("tr", {
      "data-index": String(row.index),
      "aria-selected": String(S.selected.has(row.index)),
      onclick: (ev) => selectRow(row.index, ev),
      ondblclick: () => openQuickLook(),
    });
    row.cells.forEach((value, i) => {
      const classes = [S.columns[i].kind === "money" ? "num" : "", value === "" ? "missing" : ""].join(" ").trim();
      tr.append(el("td", { class: classes }, value === "" ? "–" : value));
    });
    return tr;
  });
  $("#table tbody").replaceChildren(...rows);
  updateStatus();
}

function paintSelection() {
  for (const tr of $("#table tbody").rows) {
    tr.setAttribute("aria-selected", String(S.selected.has(Number(tr.dataset.index))));
  }
}

function selectRow(index, ev = {}) {
  const order = visibleRows().map((r) => r.index);
  if (ev.shiftKey && S.anchor !== null && order.includes(S.anchor)) {
    const [a, b] = [order.indexOf(S.anchor), order.indexOf(index)].sort((x, y) => x - y);
    S.selected = new Set(order.slice(a, b + 1));
  } else if (ev.metaKey || ev.ctrlKey) {
    if (S.selected.has(index)) S.selected.delete(index);
    else S.selected.add(index);
    S.anchor = index;
  } else {
    S.selected = new Set([index]);
    S.anchor = index;
  }
  S.active = S.selected.has(index) ? index : ([...S.selected].pop() ?? null);
  paintSelection();
  updateStatus();
  renderInspector();
}

function moveSelection(step) {
  const order = visibleRows().map((r) => r.index);
  if (!order.length) return;
  const at = order.indexOf(S.active);
  const next = order[Math.min(order.length - 1, Math.max(0, at === -1 ? 0 : at + step))];
  selectRow(next);
  $(`#table tbody tr[data-index="${next}"]`)?.scrollIntoView({ block: "nearest" });
}

/** Indizes für Exporte: die Auswahl, sonst `null` (= alle). */
function exportIndices() {
  return S.selected.size ? [...S.selected].sort((a, b) => a - b) : null;
}

function updateStatus() {
  if (!S.rows.length) return setStatus("Bereit");
  const shown = visibleRows().length;
  const parts = [shown === S.rows.length ? `${S.rows.length} ${plural(S.rows.length, "Quittung", "Quittungen")}` : `${shown} von ${S.rows.length} Quittungen`];
  if (S.selected.size) parts.push(`${S.selected.size} ausgewählt`);
  setStatus(parts.join(" · "));
}

// ---------------------------------------------------------------------------
// Schnellansicht
// ---------------------------------------------------------------------------

async function getText(index) {
  if (!S.texts.has(index)) S.texts.set(index, (await invoke("receipt_text", { index })).lines.join("\n"));
  return S.texts.get(index);
}

function cell(row, title) {
  const i = S.columns.findIndex((c) => c.title === title);
  return i === -1 ? "" : row.cells[i];
}

/** Bon-Text mit Zeilennummern (nur Ansicht, der PDF-Export bleibt unberührt). */
function showPaper(pre, text, { numbered = true } = {}) {
  pre.classList.toggle("numbered", numbered);
  if (!numbered) return void (pre.textContent = text);
  pre.replaceChildren(...text.split("\n").map((line) => el("span", { class: "ln" }, line)));
}

async function renderInspector() {
  const index = S.active;
  const row = index === null ? null : S.rows[index];
  $("#inspector-empty").hidden = row !== null;
  $("#inspector-body").hidden = row === null;
  if (!row) return;

  $("#ins-title").textContent = row.file_name;
  $("#ins-sub").textContent = cell(row, "Gerätetyp") || "Gerätetyp unbekannt";

  const range = [cell(row, "Anfangsdatum"), cell(row, "Enddatum")];
  const facts = [
    ["Zeitraum", range[0] || range[1] ? `${range[0] || "–"} – ${range[1] || "–"}` : ""],
    ["Ausdruck", cell(row, "Ausdruck")],
    ["Zulassung", cell(row, "Zulassung")],
    ["Ablauf", cell(row, "Ablauf")],
    ["Einwurf", cell(row, "Einwurf")],
    ["Auswurf", cell(row, "Auswurf")],
    ["Saldo 1", cell(row, "Saldo 1")],
    ["Saldo 2", cell(row, "Saldo 2")],
  ].filter(([, value]) => value !== "");
  $("#ins-facts").replaceChildren(...facts.flatMap(([name, value]) => [el("dt", {}, name), el("dd", {}, value)]));

  showPaper($("#ins-paper"), "Lade …", { numbered: false });
  try {
    const text = await getText(index);
    if (S.active === index) showPaper($("#ins-paper"), text);
  } catch (e) {
    if (S.active === index) showPaper($("#ins-paper"), errorText(e), { numbered: false });
  }
}

async function openQuickLook() {
  const dialog = $("#quicklook");
  if (dialog.open) return dialog.close();
  if (S.active === null) return;
  const row = S.rows[S.active];
  $("#ql-title").textContent = row.file_name;
  showPaper($("#ql-paper"), "Lade …", { numbered: false });
  dialog.showModal();
  try {
    showPaper($("#ql-paper"), await getText(row.index));
  } catch (e) {
    showPaper($("#ql-paper"), errorText(e), { numbered: false });
  }
}

// ---------------------------------------------------------------------------
// Ordner laden
// ---------------------------------------------------------------------------

function showEmpty(title, text) {
  $("#empty h1").textContent = title;
  $("#empty p").textContent = text;
  $("#empty").hidden = false;
  $("#table-wrap").hidden = true;
}

function setLoaded(loaded) {
  for (const id of ["#search", "#location", "#btn-reveal", "#btn-excel", "#btn-organize", "#btn-pdf"]) $(id).disabled = !loaded || S.busy;
}

async function chooseFolder() {
  const picked = await open({ directory: true, multiple: false, title: "Ordner mit Quittungen wählen" });
  if (picked) await loadFolder(picked);
}

async function loadFolder(path) {
  setStatus("Lese Dateien …");
  try {
    const result = await invoke("scan_folder", { path });
    Object.assign(S, {
      folder: result.folder,
      location: result.location,
      columns: result.columns,
      rows: result.rows,
      sortCol: null,
      sortDir: 1,
      selected: new Set(),
      active: null,
      anchor: null,
      texts: new Map(),
    });
    $("#search").value = "";
    $("#location").value = result.location;
    S.query = "";

    $("#folder-info").replaceChildren(
      el("strong", {}, result.location || result.folder),
      ` · ${result.rows.length} ${plural(result.rows.length, "Quittung", "Quittungen")}`,
    );

    if (result.rows.length) {
      $("#empty").hidden = true;
      $("#table-wrap").hidden = false;
      renderTable();
      selectRow(result.rows[0].index);
    } else {
      showEmpty("Keine Quittungen gefunden", "In diesem Ordner liegen keine .ACK- oder .txt-Dateien. Unterordner werden nicht durchsucht.");
      renderInspector();
      updateStatus();
    }
    setLoaded(result.rows.length > 0);

    if (result.errors.length) {
      notify({ kind: "error", title: "Einige Dateien konnten nicht gelesen werden", errors: result.errors });
    }
  } catch (e) {
    notifyError(e, "Ordner konnte nicht geladen werden");
    updateStatus();
  }
}

// ---------------------------------------------------------------------------
// Exporte
// ---------------------------------------------------------------------------

function setBusy(busy, message) {
  S.busy = busy;
  setLoaded(S.rows.length > 0);
  if (busy) setStatus(message);
  else updateStatus();
}

function scopeText() {
  return S.selected.size ? `${S.selected.size} markierte ${plural(S.selected.size, "Quittung", "Quittungen")}` : `alle ${S.rows.length} Quittungen`;
}

async function exportExcel() {
  try {
    const path = await save({
      title: "Excel-Datei speichern",
      defaultPath: `${S.folder}${sep()}${S.location}_Zusammenfassung.xlsx`,
      filters: [{ name: "Excel-Datei", extensions: ["xlsx"] }],
    });
    if (!path) return;
    setBusy(true, "Excel wird erstellt …");
    const count = await invoke("export_excel", { path, indices: exportIndices() });
    notify({ title: `${count} ${plural(count, "Quittung", "Quittungen")} als Excel gespeichert`, text: path });
  } catch (e) {
    notifyError(e, "Excel-Export fehlgeschlagen");
  } finally {
    setBusy(false);
  }
}

function currentRange() {
  const kind = $("#pdf-range").value;
  if (kind === "lines") {
    const value = parseInt($("#pdf-lines").value, 10);
    if (!(value >= 1)) throw new Error("Bitte eine Zeilenzahl ab 1 eingeben.");
    return { kind, value };
  }
  if (kind === "keyword") {
    const value = $("#pdf-keyword").value.trim();
    if (!value) throw new Error("Bitte ein Stichwort eingeben.");
    const occurrence = parseInt($("#pdf-occ").value, 10) || 1;
    if (occurrence < 1) throw new Error("Die Nennung muss mindestens 1 sein.");
    return { kind, value, occurrence, offset: parseInt($("#pdf-offset").value, 10) || 0 };
  }
  return { kind };
}

async function exportPdfs() {
  let range;
  try {
    range = currentRange();
  } catch (e) {
    return notifyError(e, "PDF-Optionen unvollständig");
  }
  setBusy(true, "PDFs werden erstellt …");
  try {
    const report = await invoke("export_pdfs", { range, indices: exportIndices(), outDir: null, template: S.template });
    const n = report.written.length;
    notify({
      kind: n === 0 ? "error" : "ok",
      title: `${n} ${plural(n, "PDF", "PDFs")} erstellt`,
      text: `Ordner: ${report.out_dir}`,
      warnings: report.warnings,
      errors: report.errors,
    });
  } catch (e) {
    notifyError(e, "PDF-Export fehlgeschlagen");
  } finally {
    setBusy(false);
  }
}

function updatePdfInputs() {
  const kind = $("#pdf-range").value;
  $("#pdf-lines").hidden = kind !== "lines";
  $("#pdf-keyword").hidden = kind !== "keyword";
  $("#wrap-occ").hidden = kind !== "keyword";
  $("#wrap-offset").hidden = kind !== "keyword";
}

// ---------------------------------------------------------------------------
// Umbenennen
// ---------------------------------------------------------------------------

const orgMode = () => document.querySelector('input[name="org-mode"]:checked').value;

function updateOrganizeTexts() {
  const move = orgMode() === "move";
  $("#org-explain").textContent = move
    ? `Die Originale werden nach ${S.outDir} verschoben und verschwinden aus dem aktuellen Ordner.`
    : `Die Originale bleiben liegen. Kopien entstehen in ${S.outDir}.`;
  $("#org-go").classList.toggle("danger", move);
  $("#org-go").classList.toggle("primary", !move);
  $("#org-go").textContent = move ? "Verschieben" : "Kopieren";
}

async function openOrganize() {
  const dialog = $("#organize");
  dialog.returnValue = "";
  document.querySelector(`input[name="org-mode"][value="${store.get("mode", "copy")}"]`).checked = true;
  $("#org-scope").textContent = `Betrifft ${scopeText()}. Dateien, die es am Ziel schon gibt, werden nie überschrieben.`;
  S.outDir = await invoke("output_dir");
  updateOrganizeTexts();
  showPreview($("#org-example"), S.template, S.active, "ACK");
  dialog.showModal();
}

async function runOrganize() {
  const mode = orgMode();
  setBusy(true, mode === "move" ? "Dateien werden verschoben …" : "Dateien werden kopiert …");
  try {
    const report = await invoke("organize_files", { mode, template: S.template, indices: exportIndices(), outDir: null });
    const n = report.written.length;
    notify({
      kind: n === 0 ? "error" : "ok",
      title: `${n} ${plural(n, "Datei", "Dateien")} ${mode === "move" ? "verschoben" : "kopiert"}`,
      text: `Ordner: ${report.out_dir}`,
      errors: report.errors,
    });
  } catch (e) {
    notifyError(e, "Umbenennen fehlgeschlagen");
  } finally {
    setBusy(false);
  }
}

// ---------------------------------------------------------------------------
// Einstellungen: Dateiname-Vorlage mit Live-Vorschau
// ---------------------------------------------------------------------------

let previewToken = 0;

/** Schreibt die Vorschau oder den Fehlertext (rot) in `box`. */
async function showPreview(box, template, index, extension) {
  const token = ++previewToken;
  let text;
  let failed = false;
  try {
    text = await invoke("preview_filename", { template, index, extension });
  } catch (e) {
    text = errorText(e);
    failed = true;
  }
  if (token !== previewToken) return; // inzwischen neuere Eingabe
  box.textContent = text;
  box.classList.toggle("error", failed);
}

function openSettings() {
  const dialog = $("#settings");
  $("#tpl").value = S.template;
  document.querySelector(`input[name="default-mode"][value="${store.get("mode", "copy")}"]`).checked = true;
  showPreview($("#tpl-preview"), S.template, S.active, "ACK");
  dialog.showModal();
  $("#tpl").focus();
}

function setTemplate(value) {
  S.template = value;
  store.set("template", value);
  showPreview($("#tpl-preview"), value, S.active, "ACK");
}

function buildChips() {
  $("#chips").replaceChildren(
    ...S.placeholders.map((p) =>
      el(
        "button",
        {
          class: "chip",
          type: "button",
          title: p.description,
          onclick: () => {
            const input = $("#tpl");
            const start = input.selectionStart ?? input.value.length;
            const end = input.selectionEnd ?? start;
            input.setRangeText(`{${p.name}}`, start, end, "end");
            input.focus();
            setTemplate(input.value);
          },
        },
        `{${p.name}}`,
      ),
    ),
  );
}

// ---------------------------------------------------------------------------
// Start
// ---------------------------------------------------------------------------

function anyDialogOpen() {
  return document.querySelector("dialog[open]") !== null;
}

function wireEvents() {
  $("#btn-open").addEventListener("click", chooseFolder);
  $("#btn-open-empty").addEventListener("click", chooseFolder);
  if (!isMac) $("#btn-reveal").textContent = "Im Explorer zeigen";
  $("#btn-reveal").addEventListener("click", () => invoke("open_folder").catch((e) => notifyError(e, "Ordner konnte nicht geöffnet werden")));
  $("#btn-excel").addEventListener("click", exportExcel);
  $("#btn-pdf").addEventListener("click", exportPdfs);
  $("#btn-organize").addEventListener("click", openOrganize);
  $("#btn-settings").addEventListener("click", openSettings);
  $("#btn-quicklook").addEventListener("click", openQuickLook);
  $("#ql-close").addEventListener("click", () => $("#quicklook").close());
  $("#notice-close").addEventListener("click", () => ($("#notice").hidden = true));

  // Aufstellort überschreiben; leer = Ordnername
  $("#location").addEventListener("change", async (ev) => {
    S.location = await invoke("set_location", { location: ev.target.value });
    ev.target.value = S.location;
    const info = $("#folder-info strong");
    if (info) info.textContent = S.location;
  });

  $("#search").addEventListener("input", (ev) => {
    S.query = ev.target.value;
    renderTable();
  });

  // PDF-Optionen merken
  $("#pdf-range").value = store.get("pdfRange", "all");
  $("#pdf-lines").value = store.get("pdfLines", 20);
  $("#pdf-keyword").value = store.get("pdfKeyword", "");
  $("#pdf-occ").value = store.get("pdfOcc", 1);
  $("#pdf-offset").value = store.get("pdfOffset", 0);
  updatePdfInputs();
  $("#pdf-range").addEventListener("change", () => {
    store.set("pdfRange", $("#pdf-range").value);
    updatePdfInputs();
  });
  $("#pdf-lines").addEventListener("change", () => store.set("pdfLines", $("#pdf-lines").value));
  $("#pdf-keyword").addEventListener("change", () => store.set("pdfKeyword", $("#pdf-keyword").value));
  $("#pdf-occ").addEventListener("change", () => store.set("pdfOcc", $("#pdf-occ").value));
  $("#pdf-offset").addEventListener("change", () => store.set("pdfOffset", $("#pdf-offset").value));

  // Einstellungen
  $("#tpl").addEventListener("input", (ev) => setTemplate(ev.target.value));
  $("#tpl-reset").addEventListener("click", () => {
    $("#tpl").value = S.defaultTemplate;
    setTemplate(S.defaultTemplate);
  });
  for (const radio of document.querySelectorAll('input[name="default-mode"]')) {
    radio.addEventListener("change", () => store.set("mode", radio.value));
  }

  // Umbenennen bestätigen
  for (const radio of document.querySelectorAll('input[name="org-mode"]')) radio.addEventListener("change", updateOrganizeTexts);
  $("#org-cancel").addEventListener("click", () => $("#organize").close("cancel"));
  $("#organize").addEventListener("close", () => {
    if ($("#organize").returnValue === "go") runOrganize();
  });

  // Tastatur: ⌘O, ⌘,, Pfeile, Leertaste (Quick Look), ⌘A
  document.addEventListener("keydown", (ev) => {
    const mod = ev.metaKey || ev.ctrlKey;
    if (mod && ev.key === "o") {
      ev.preventDefault();
      if (!anyDialogOpen()) chooseFolder();
      return;
    }
    if (mod && ev.key === ",") {
      ev.preventDefault();
      if (!anyDialogOpen()) openSettings();
      return;
    }

    const typing = ["INPUT", "SELECT", "TEXTAREA"].includes(document.activeElement?.tagName);
    if (ev.key === " " && $("#quicklook").open) {
      ev.preventDefault();
      return $("#quicklook").close();
    }
    if (typing || anyDialogOpen() || !S.rows.length) return;

    if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      moveSelection(ev.key === "ArrowDown" ? 1 : -1);
    } else if (ev.key === " " && S.active !== null) {
      ev.preventDefault();
      openQuickLook();
    } else if (mod && ev.key === "a") {
      ev.preventDefault();
      S.selected = new Set(visibleRows().map((r) => r.index));
      paintSelection();
      updateStatus();
    }
  });

  window.addEventListener("unhandledrejection", (ev) => notifyError(ev.reason));
}

async function init() {
  wireEvents();
  try {
    const info = await invoke("template_info");
    S.defaultTemplate = info.default_template;
    S.placeholders = info.placeholders;
    S.template = store.get("template", info.default_template);
    buildChips();

    // Nur für Entwicklung: MAS_OPEN_FOLDER=<Pfad> lädt den Ordner beim Start
    const initial = await invoke("initial_folder");
    if (initial) await loadFolder(initial);
  } catch (e) {
    notifyError(e, "Start fehlgeschlagen");
  }
}

init();
