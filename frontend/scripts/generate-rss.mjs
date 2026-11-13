#!/usr/bin/env node
/**
 * generate-rss.mjs
 *
 * Generates a static RSS 2.0 feed from the changelog markdown files
 * located at content/changelog/*.md and writes the result to
 * public/rss.xml so it is served as a plain static asset.
 *
 * Usage:
 *   node scripts/generate-rss.mjs
 *
 * Also wired into the `prebuild` npm script so it runs automatically
 * before every `next build`.
 *
 * Environment variables:
 *   NEXT_PUBLIC_SITE_URL  – Deployed origin (default: https://soroban-zk-std.dev)
 */

import { readFileSync, writeFileSync, readdirSync, existsSync, mkdirSync } from 'fs';
import { resolve, dirname, join } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname  = dirname(__filename);

const FRONTEND_ROOT  = resolve(__dirname, '..');
const CHANGELOG_DIR  = join(FRONTEND_ROOT, 'content', 'changelog');
const PUBLIC_DIR     = join(FRONTEND_ROOT, 'public');
const OUT_FILE       = join(PUBLIC_DIR, 'rss.xml');

const SITE_URL   = (process.env.NEXT_PUBLIC_SITE_URL ?? 'https://soroban-zk-std.dev').replace(/\/$/, '');
const FEED_URL   = `${SITE_URL}/rss.xml`;
const GITHUB_REPO = 'https://github.com/cyberdocs120/stellar-zk-std';

// ---------------------------------------------------------------------------
// Front-matter parser
// ---------------------------------------------------------------------------

/**
 * Parse a minimal YAML front-matter block.
 * Supports scalar strings and inline arrays only — no nesting.
 *
 * @param {string} raw  Raw file contents.
 * @returns {{ meta: Record<string, unknown>, body: string }}
 */
function parseFrontMatter(raw) {
  const FM_RE = /^---\r?\n([\s\S]*?)\r?\n---\r?\n?([\s\S]*)$/;
  const match = raw.match(FM_RE);
  if (!match) return { meta: {}, body: raw.trim() };

  const [, yamlBlock, body] = match;
  /** @type {Record<string, unknown>} */
  const meta = {};

  for (const line of yamlBlock.split(/\r?\n/)) {
    const colonIdx = line.indexOf(':');
    if (colonIdx === -1) continue;

    const key      = line.slice(0, colonIdx).trim();
    const rawValue = line.slice(colonIdx + 1).trim();

    if (rawValue.startsWith('[') && rawValue.endsWith(']')) {
      meta[key] = rawValue
        .slice(1, -1)
        .split(',')
        .map((s) => s.trim().replace(/^["']|["']$/g, ''))
        .filter(Boolean);
    } else {
      meta[key] = rawValue.replace(/^["']|["']$/g, '');
    }
  }

  return { meta, body: body.trim() };
}

// ---------------------------------------------------------------------------
// Changelog loader
// ---------------------------------------------------------------------------

/**
 * @typedef {{ title: string, date: string, version: string, author: string,
 *             commit?: string, body: string, filename: string }} ChangelogEntry
 */

/**
 * Load and return all changelog entries sorted newest-first.
 * @returns {ChangelogEntry[]}
 */
function getChangelogEntries() {
  if (!existsSync(CHANGELOG_DIR)) {
    console.warn(`[generate-rss] Changelog directory not found: ${CHANGELOG_DIR}`);
    return [];
  }

  const files = readdirSync(CHANGELOG_DIR)
    .filter((f) => f.endsWith('.md') || f.endsWith('.mdx'))
    .sort()
    .reverse();

  const entries = files.map((filename) => {
    const filePath = join(CHANGELOG_DIR, filename);
    const raw      = readFileSync(filePath, 'utf-8');
    const { meta, body } = parseFrontMatter(raw);

    return {
      title:    String(meta.title   ?? filename),
      date:     String(meta.date    ?? ''),
      version:  String(meta.version ?? 'patch'),
      author:   String(meta.author  ?? 'unknown'),
      commit:   meta.commit ? String(meta.commit) : undefined,
      body,
      filename,
    };
  });

  return entries.sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime()
  );
}

// ---------------------------------------------------------------------------
// XML helpers
// ---------------------------------------------------------------------------

/** @param {string} str */
const escapeXml = (str) =>
  str
    .replace(/&/g,  '&amp;')
    .replace(/</g,  '&lt;')
    .replace(/>/g,  '&gt;')
    .replace(/"/g,  '&quot;')
    .replace(/'/g,  '&apos;');

/**
 * "YYYY-MM-DD" → RFC 822 string required by RSS <pubDate>.
 * @param {string} dateStr
 */
function toRfc822(dateStr) {
  const d = new Date(dateStr);
  return isNaN(d.getTime()) ? new Date().toUTCString() : d.toUTCString();
}

/**
 * Minimal Markdown → HTML for the <description> CDATA block.
 * Handles paragraphs, bullet lists, inline code, bold, and links.
 * @param {string} md
 */
function markdownToHtml(md) {
  const lines = md.split(/\r?\n/);
  const chunks = [];
  let inList = false;

  const inlineFormat = (text) =>
    text
      .replace(/&/g,  '&amp;')
      .replace(/</g,  '&lt;')
      .replace(/>/g,  '&gt;')
      .replace(/`([^`]+)`/g,          '<code>$1</code>')
      .replace(/\*\*([^*]+)\*\*/g,    '<strong>$1</strong>')
      .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith('- ')) {
      if (!inList) { chunks.push('<ul>'); inList = true; }
      chunks.push(`<li>${inlineFormat(trimmed.slice(2))}</li>`);
    } else {
      if (inList) { chunks.push('</ul>'); inList = false; }
      chunks.push(trimmed === '' ? '' : `<p>${inlineFormat(trimmed)}</p>`);
    }
  }
  if (inList) chunks.push('</ul>');

  return chunks
    .filter((c, i) => !(c === '' && chunks[i - 1] === ''))
    .join('\n');
}

// ---------------------------------------------------------------------------
// Feed builder
// ---------------------------------------------------------------------------

function buildFeed() {
  const entries       = getChangelogEntries();
  const lastBuildDate = entries.length > 0
    ? toRfc822(entries[0].date)
    : new Date().toUTCString();

  const items = entries
    .map((entry) => {
      const link = entry.commit
        ? `${GITHUB_REPO}/commit/${entry.commit}`
        : GITHUB_REPO;

      return `
    <item>
      <title>${escapeXml(entry.title)}</title>
      <link>${escapeXml(link)}</link>
      <guid isPermaLink="true">${escapeXml(link)}</guid>
      <pubDate>${toRfc822(entry.date)}</pubDate>
      <dc:creator>${escapeXml(entry.author)}</dc:creator>
      <description><![CDATA[${markdownToHtml(entry.body)}]]></description>
    </item>`;
    })
    .join('\n');

  return `<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"
  xmlns:atom="http://www.w3.org/2005/Atom"
  xmlns:dc="http://purl.org/dc/elements/1.1/">
  <channel>
    <title>Soroban-ZK-Std Changelog</title>
    <link>${escapeXml(GITHUB_REPO)}</link>
    <description>Updates to Soroban-ZK-Std — a high-performance cryptographic standard library for Stellar Protocol 25 ZK-primitives (BN254, Poseidon2, ElGamal, Pairing).</description>
    <language>en-us</language>
    <copyright>Copyright 2026 Soroban-ZK-Std contributors. Apache-2.0 licensed.</copyright>
    <lastBuildDate>${lastBuildDate}</lastBuildDate>
    <atom:link href="${escapeXml(FEED_URL)}" rel="self" type="application/rss+xml"/>
${items}
  </channel>
</rss>`;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

function main() {
  // Ensure public/ exists
  if (!existsSync(PUBLIC_DIR)) {
    mkdirSync(PUBLIC_DIR, { recursive: true });
  }

  const xml = buildFeed();
  writeFileSync(OUT_FILE, xml, 'utf-8');

  const entries = getChangelogEntries();
  console.log(`[generate-rss] rss.xml written → ${OUT_FILE}`);
  console.log(`[generate-rss] ${entries.length} changelog entries included.`);
}

main();
