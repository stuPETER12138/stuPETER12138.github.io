import { Cite, plugins } from '@citation-js/core';
import '@citation-js/plugin-bibtex';

function httpUrl(value) {
  if (!value) return '';
  try {
    const url = new URL(value);
    return ['http:', 'https:'].includes(url.protocol) ? url.href : '';
  } catch { return ''; }
}

export function parsePublications(source) {
  if (!source.trim()) return [];
  const raw = plugins.input.chain(source, { target: '@biblatex/entries+list' });
  const keys = new Set();
  for (const entry of raw) {
    if (keys.has(entry.label)) throw new Error(`Duplicate BibTeX key: ${entry.label}`);
    keys.add(entry.label);
  }
  const fields = new Map(raw.map(entry => [entry.label, entry.properties]));
  return new Cite(source).data.map(entry => {
    const key = entry['citation-key'] || entry.id;
    const original = fields.get(key) || {};
    if (!entry.title) throw new Error(`Missing BibTeX title: ${key}`);
    // A citation key is an identifier, not an implicit DOI.
    const doi = String(entry.DOI || '').replace(/^https?:\/\/(?:dx\.)?doi\.org\//i, '');
    const doiUrl = /^10\.\d{4,9}\/\S+$/i.test(doi) ? `https://doi.org/${doi}` : '';
    const arxiv = /^(arxiv)$/i.test(original.archiveprefix || original.eprinttype || '')
      ? String(original.eprint || '') : '';
    const arxivUrl = /^(?:\d{4}\.\d{4,5}|[a-z.-]+\/\d{7})(?:v\d+)?$/i.test(arxiv)
      ? `https://arxiv.org/abs/${arxiv}` : '';
    const url = httpUrl(entry.URL) || doiUrl || arxivUrl;
    const links = [];
    if (doiUrl) links.push({ label: 'DOI', url: doiUrl });
    if (arxivUrl && arxivUrl !== doiUrl) links.push({ label: 'arXiv', url: arxivUrl });
    if (url && !links.some(link => link.url === url)) links.push({ label: '论文', url });
    const authorNames = (entry.author || []).map(author => author.literal ||
      [author.given, author['non-dropping-particle'], author.family, author.suffix].filter(Boolean).join(' '));
    // Published journal/conference types take precedence over an accompanying preprint link.
    const typeLabel = entry.type === 'article-journal' ? '期刊论文'
      : entry.type === 'paper-conference' ? '会议论文'
      : arxivUrl ? '预印本' : '学术成果';
    return {
      key,
      year: entry.issued?.['date-parts']?.[0]?.[0] || '',
      title: entry.title,
      authors: authorNames.join(', '),
      authorNames,
      typeLabel,
      pages: String(entry.page || '').replace(/--|(?<=\d)-(?=\d)/g, '–'),
      publisher: entry.publisher || '',
      venue: entry['container-title'] || (arxivUrl ? `arXiv:${arxiv} · 预印本` : entry.publisher || ''),
      note: entry.note || '',
      abstract: entry.abstract || '',
      url,
      links
    };
  }).sort((a, b) => Number(b.year) - Number(a.year));
}
