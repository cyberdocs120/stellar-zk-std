import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

async function fetchVersions() {
  try {
    console.log('Fetching tags from GitHub...');
    const response = await fetch('https://api.github.com/repos/cyberdocs120/stellar-zk-std/tags', {
      headers: { 'User-Agent': 'Soroban-ZK-Docs-Script' }
    });
    
    let tags = [];
    if (response.ok) {
      tags = await response.json();
    } else {
      console.warn('Failed to fetch from GitHub, status:', response.status);
    }

    let versions = [];
    if (tags.length === 0) {
      console.log('No tags found on GitHub. Falling back to local Cargo.toml version...');
      // Fallback: read Cargo.toml
      const cargoPath = path.join(__dirname, '..', '..', 'crates', 'soroban-zk-std', 'Cargo.toml');
      const cargoContent = fs.readFileSync(cargoPath, 'utf8');
      const match = cargoContent.match(/version\s*=\s*"([^"]+)"/);
      const version = match ? `v${match[1]}` : 'v0.1.0';
      
      versions = [
        {
          id: version,
          name: version,
          label: `Latest (${version})`,
          isLatest: true,
        }
      ];
    } else {
      versions = tags.map((tag, index) => ({
        id: tag.name,
        name: tag.name,
        label: index === 0 ? `Latest (${tag.name})` : tag.name,
        isLatest: index === 0,
      }));
    }

    const outputPath = path.join(__dirname, '..', 'data', 'versions.json');
    fs.writeFileSync(outputPath, JSON.stringify(versions, null, 2));
    console.log(`versions.json written successfully with ${versions.length} versions.`);
  } catch (error) {
    console.error('Error fetching versions:', error);
    process.exit(1);
  }
}

fetchVersions();
