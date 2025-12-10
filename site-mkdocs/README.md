# AirsSys Unified Documentation Site

This directory contains the MkDocs configuration for the unified AirsSys documentation.

## Quick Start

### Local Development

```bash
# Install dependencies
pip install -r requirements.txt

# Serve locally (with live reload)
mkdocs serve

# Visit http://localhost:8000
```

### Build Static Site

```bash
# Build site to site/ directory
mkdocs build

# Build with verbose output
mkdocs build --verbose
```

### Deploy to GitHub Pages

```bash
# Deploy manually
mkdocs gh-deploy --force --clean

# Or push to main branch to trigger GitHub Actions
git push origin main
```

## Structure

```
site-mkdocs/
├── mkdocs.yml          # Main configuration
├── requirements.txt    # Python dependencies
├── overrides/          # Theme customization
│   ├── main.html      # Template overrides
│   └── assets/
│       └── extra.css  # Custom CSS
└── site/              # Build output (gitignored)
```

## Configuration

### Key Settings

- **Theme**: Material for MkDocs
- **Colors**: Deep Orange (primary), Teal (accent)
- **Features**: Navigation tabs, search, code copy, dark mode
- **Extensions**: Mermaid diagrams, syntax highlighting, admonitions

### Navigation

Navigation structure is defined in `mkdocs.yml` under the `nav` key. The structure follows:

- Home (Welcome, Overview, Getting Started, Architecture)
- Components (OSL, RT with full documentation hierarchy)
- Guides (Integration, Security, Performance)
- Examples (OSL, RT examples)
- Research (RT research documentation)
- Contributing

## Documentation Source

Documentation source files are located in `../docs/` (relative to this directory).

```
docs/
├── index.md                  # Landing page
├── overview.md              # Ecosystem overview
├── getting-started.md       # Installation guide
├── architecture.md          # System architecture
├── components/              # Component docs
│   ├── osl/                # OSL documentation (32 files)
│   └── rt/                 # RT documentation (47 files)
├── guides/                  # Cross-component guides
├── examples/                # Usage examples
├── research/                # Research documentation
└── contributing.md          # Contribution guidelines
```

## Deployment

### Automatic (GitHub Actions)

The site automatically deploys when changes are pushed to `main` branch:

1. GitHub Actions detects changes in `docs/` or `site-mkdocs/`
2. Workflow runs `mkdocs build`
3. Site deployed to `gh-pages` branch
4. GitHub Pages serves the site

### Manual

```bash
# From site-mkdocs directory
mkdocs gh-deploy --force --clean --verbose
```

## URL

Production documentation: https://airsstack.github.io/airssys/

## Maintenance

### Adding New Pages

1. Create markdown file in `docs/`
2. Add entry to `nav` in `mkdocs.yml`
3. Test locally with `mkdocs serve`
4. Commit and push

### Updating Theme

Theme settings are in `mkdocs.yml` under the `theme` key.

Custom CSS: `overrides/assets/extra.css`  
Custom HTML: `overrides/main.html`

### Troubleshooting

**Build fails:**
```bash
# Check for syntax errors
mkdocs build --verbose

# Validate navigation
# Ensure all nav entries point to existing files
```

**Links broken:**
```bash
# Check warning messages during build
mkdocs build 2>&1 | grep WARNING
```

**Site not updating:**
```bash
# Clear cache and rebuild
mkdocs build --clean
```

## Dependencies

See `requirements.txt`:
- mkdocs >=1.5.0
- mkdocs-material >=9.5.0
- pymdown-extensions >=10.7.0

## Resources

- **MkDocs**: https://www.mkdocs.org
- **Material for MkDocs**: https://squidfunk.github.io/mkdocs-material/
- **Deployment Guide**: ../DEPLOYMENT.md

---

For full deployment instructions, see [DEPLOYMENT.md](../DEPLOYMENT.md)
