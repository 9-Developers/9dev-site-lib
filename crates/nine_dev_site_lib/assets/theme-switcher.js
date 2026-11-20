(() => {
  const keys = new Set(["system", "latte", "frappe", "macchiato", "mocha"]);
  const normalize = value => keys.has(value) ? value : "system";
  const media = matchMedia("(prefers-color-scheme: dark)");
  let preference = "system";
  try { preference = normalize(localStorage.getItem("theme")); } catch {}

  function apply() {
    document.documentElement.dataset.theme = preference === "system"
      ? (media.matches ? "mocha" : "latte") : preference;
    for (const menu of document.querySelectorAll(".nd-theme")) {
      const summary = menu.querySelector("summary");
      for (const option of menu.querySelectorAll("button[data-theme-value]")) {
        const selected = option.dataset.themeValue === preference;
        option.setAttribute("aria-pressed", String(selected));
        if (selected) {
          summary.querySelector("use").setAttribute("href", option.querySelector("use").getAttribute("href"));
          summary.setAttribute("aria-label", `Color theme: ${option.querySelector("span").textContent}`);
        }
      }
    }
  }

  apply();
  document.addEventListener("DOMContentLoaded", apply, { once: true });
  media.addEventListener("change", apply);
  window.addEventListener("storage", event => {
    if (event.key === "theme" || event.key === null) {
      preference = normalize(event.newValue);
      apply();
    }
  });
  function close(menu, focus) {
    menu.open = false;
    if (focus) menu.querySelector("summary").focus();
  }
  document.addEventListener("click", event => {
    const option = event.target.closest?.(".nd-theme button[data-theme-value]");
    if (option) {
      preference = normalize(option.dataset.themeValue);
      try {
        if (preference === "system") localStorage.removeItem("theme");
        else localStorage.setItem("theme", preference);
      } catch {}
      apply();
      close(option.closest(".nd-theme"), true);
    }
    for (const menu of document.querySelectorAll(".nd-theme[open]")) {
      if (!menu.contains(event.target)) close(menu, false);
    }
  });
  document.addEventListener("keydown", event => {
    const menu = event.target.closest?.(".nd-theme[open]");
    if (menu && event.key === "Escape") {
      event.preventDefault();
      close(menu, true);
    }
  });
  document.addEventListener("focusout", event => {
    const menu = event.target.closest?.(".nd-theme[open]");
    if (menu && event.relatedTarget && !menu.contains(event.relatedTarget)) close(menu, false);
  });
})();
