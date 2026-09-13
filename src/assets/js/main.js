const root = document.documentElement;
const toggle = document.querySelector('.theme-toggle');
const preference = window.matchMedia('(prefers-color-scheme: dark)');
let savedTheme;
try { savedTheme = localStorage.getItem('academic-theme'); } catch { /* Storage may be disabled. */ }
if (savedTheme === 'light' || savedTheme === 'dark') root.dataset.theme = savedTheme;
const isDark = () => root.dataset.theme ? root.dataset.theme === 'dark' : preference.matches;
function updateLabel() {
  const label = `切换至${isDark() ? '浅色' : '深色'}主题`;
  toggle.setAttribute('aria-label', label);
  toggle.title = label;
}
if (toggle) {
  toggle.hidden = false;
  updateLabel();
  preference.addEventListener('change', updateLabel);
  toggle.addEventListener('click', () => {
    root.dataset.theme = isDark() ? 'light' : 'dark';
    try { localStorage.setItem('academic-theme', root.dataset.theme); } catch { /* Theme still works without persistence. */ }
    updateLabel();
  });
}
