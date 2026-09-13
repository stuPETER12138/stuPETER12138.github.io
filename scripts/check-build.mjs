import assert from 'node:assert/strict';
import { readFile, access } from 'node:fs/promises';
import path from 'node:path';
import { parsePublications } from './publications.mjs';

const output = path.resolve('_site');
const html = await readFile(path.join(output, 'index.html'), 'utf8');
assert.match(html, /<html lang="zh-CN">/);
assert.match(html, /rel="canonical"/);
const papers = parsePublications(await readFile('papers.bib', 'utf8'));
assert.equal([...html.matchAll(/<article class="publication">/g)].length, papers.length, 'All BibTeX entries must render');
const escapeHtml = value => String(value).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
for (const paper of papers) {
  assert.ok(html.includes(escapeHtml(paper.title)), `Missing paper title: ${paper.key}`);
  assert.ok(html.includes(escapeHtml(paper.authors)), `Missing authors: ${paper.key}`);
  assert.ok(html.includes(escapeHtml(paper.typeLabel)), `Missing type: ${paper.key}`);
  if (paper.pages) assert.ok(html.includes(`pp. ${escapeHtml(paper.pages)}`), `Missing pages: ${paper.key}`);
}
assert.ok(!html.includes('{{') && !html.includes('{%'), 'Unresolved template expressions');
for (const id of ['main', 'about', 'news', 'research', 'publications', 'experience', 'contact']) {
  assert.ok(html.includes(`id="${id}"`), `Missing section: ${id}`);
}
for (const file of ['404.html', 'sitemap.xml', 'robots.txt', 'assets/css/style.css', 'assets/js/main.js', 'assets/favicon.svg']) {
  await access(path.join(output, file));
}
for (const [, value] of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
  if (/^(?:https?:|mailto:|tel:|data:)/.test(value)) continue;
  const [pathname, fragment] = value.split('#');
  if (fragment && (!pathname || pathname === '/')) {
    assert.ok(html.includes(`id="${fragment}"`), `Broken anchor: ${value}`);
  }
  if (pathname) {
    const local = decodeURIComponent(pathname).replace(/^\//, '');
    await access(path.join(output, local || 'index.html'));
  }
}
assert.equal(await readFile(path.join(output, 'papers.bib'), 'utf8'), await readFile('papers.bib', 'utf8'), 'Download must match source bibliography');
const sitemap = await readFile(path.join(output, 'sitemap.xml'), 'utf8');
assert.ok(sitemap.trimStart().startsWith('<?xml'));
console.log('Build checks passed: sections, metadata, assets, local links and sitemap.');
