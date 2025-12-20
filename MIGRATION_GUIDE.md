# Guide de Migration vers kzn-tools

## Étape 1 : Créer l'Organisation (Maintenant)

### 1.1 Créer l'organisation GitHub

1. Aller sur https://github.com/organizations/plan
2. Choisir **Free** (suffisant pour commencer)
3. Nom : `kzn-tools`
4. Email de contact : votre email

### 1.2 Créer les 4 repositories

| Ordre | Nom | Visibilité | Description |
|-------|-----|------------|-------------|
| 1 | `kaizen` | Public | Ultra-fast JavaScript/TypeScript static analyzer |
| 2 | `zed` | Public | Kaizen extension for Zed editor |
| 3 | `vscode` | Public | Kaizen extension for VS Code |
| 4 | `cloud` | **Private** | Kaizen SaaS backend |

**Important :** Créer les repos VIDES (sans README, sans LICENSE).

---

## Étape 2 : Transférer le Repo Principal (Quand Prêt)

### Option A : Transférer le repo existant (Recommandé)

```
github.com/mpiton/kaizen → github.com/kzn-tools/kaizen
```

1. Aller sur https://github.com/mpiton/kaizen/settings
2. Scroll → "Danger Zone" → "Transfer ownership"
3. Nouveau owner : `kzn-tools`
4. Confirmer

**Avantages :**
- Garde l'historique git complet
- Les stars et forks sont préservés
- Les redirections automatiques fonctionnent

### Option B : Push vers nouveau repo

```bash
# Ajouter le nouveau remote
git remote add kzn git@github.com:kzn-tools/kaizen.git

# Push
git push kzn main --tags

# Changer le remote origin
git remote set-url origin git@github.com:kzn-tools/kaizen.git
```

---

## Étape 3 : Extraire les Extensions (Phase 2)

### 3.1 Extraire l'extension Zed

```bash
# Créer un nouveau dossier
mkdir -p ~/projets/kzn-zed
cd ~/projets/kzn-zed

# Copier les fichiers
cp -r ~/projets/lynx/editors/zed/* .

# Initialiser git
git init
git add .
git commit -m "feat: extract Zed extension from monorepo"

# Push vers kzn-tools/zed
git remote add origin git@github.com:kzn-tools/zed.git
git push -u origin main
```

### 3.2 Mettre à jour extension.toml

```toml
id = "kaizen"
name = "Kaizen"
version = "0.1.0"
schema_version = 1
authors = ["mpiton <contact@mpiton.dev>"]
description = "Ultra-fast JavaScript/TypeScript static analyzer"
repository = "https://github.com/kzn-tools/zed"  # ← CHANGÉ

[lib]
kind = "Rust"
version = "0.7.0"

[language_servers.kaizen-lsp]
name = "Kaizen"
languages = ["JavaScript", "TypeScript"]
```

### 3.3 Extraire l'extension VS Code

```bash
mkdir -p ~/projets/kzn-vscode
cd ~/projets/kzn-vscode

cp -r ~/projets/lynx/editors/vscode/* .

git init
git add .
git commit -m "feat: extract VS Code extension from monorepo"

git remote add origin git@github.com:kzn-tools/vscode.git
git push -u origin main
```

### 3.4 Nettoyer le repo principal

```bash
cd ~/projets/lynx

# Supprimer les dossiers editors
rm -rf editors/

# Ajouter un lien vers les nouveaux repos
mkdir -p editors
echo "Extensions moved to separate repositories:
- Zed: https://github.com/kzn-tools/zed
- VS Code: https://github.com/kzn-tools/vscode" > editors/README.md

git add .
git commit -m "chore: move editor extensions to separate repos"
git push
```

---

## Étape 4 : Mettre à Jour les Références

### 4.1 Fichiers à modifier dans `kzn-tools/kaizen`

| Fichier | Changement |
|---------|------------|
| `Cargo.toml` | `repository = "https://github.com/kzn-tools/kaizen"` |
| `README.md` | Mettre à jour les URLs et badges |
| `.github/workflows/*.yml` | Mettre à jour si nécessaire |
| `action.yml` | `uses: kzn-tools/kaizen@main` |

### 4.2 Cargo.toml (workspace)

```toml
[workspace.package]
version = "0.1.0"
edition = "2024"
authors = ["Matvei Piton <mpiton@users.noreply.github.com>"]
license = "MIT"
repository = "https://github.com/kzn-tools/kaizen"  # ← CHANGÉ
description = "Ultra-fast JavaScript/TypeScript static analyzer written in Rust"
```

### 4.3 README.md badges

```markdown
[![CI](https://github.com/kzn-tools/kaizen/actions/workflows/ci.yml/badge.svg)](https://github.com/kzn-tools/kaizen/actions/workflows/ci.yml)
```

### 4.4 GitHub Action usage

```yaml
- name: Run Kaizen
  uses: kzn-tools/kaizen@main  # ← CHANGÉ
```

---

## Étape 5 : Créer le Repo Cloud (Privé)

```bash
mkdir -p ~/projets/kzn-cloud
cd ~/projets/kzn-cloud

# Structure initiale
mkdir -p api/src dashboard/src infra

# Créer Cargo.toml pour l'API
cat > api/Cargo.toml << 'EOF'
[package]
name = "kaizen-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio"] }
EOF

git init
git add .
git commit -m "feat: initial cloud backend structure"

git remote add origin git@github.com:kzn-tools/cloud.git
git push -u origin main
```

---

## Checklist de Migration

### Phase 1 : Préparation (Maintenant)
- [ ] Créer l'organisation `kzn-tools` sur GitHub
- [ ] Créer les 4 repos vides (kaizen, zed, vscode, cloud)
- [ ] Inviter les collaborateurs à l'organisation

### Phase 2 : Transfert (Quand Prêt pour SaaS)
- [ ] Transférer `mpiton/kaizen` → `kzn-tools/kaizen`
- [ ] Mettre à jour les URLs dans Cargo.toml
- [ ] Mettre à jour les badges dans README.md
- [ ] Extraire editors/zed → `kzn-tools/zed`
- [ ] Extraire editors/vscode → `kzn-tools/vscode`
- [ ] Créer la structure initiale de `kzn-tools/cloud`

### Phase 3 : Vérification
- [ ] CI/CD fonctionne sur le nouveau repo
- [ ] npm publish fonctionne
- [ ] Extensions publiées sur les marketplaces
- [ ] Documentation mise à jour

---

## URLs Finales

| Composant | URL Actuelle | URL Finale |
|-----------|--------------|------------|
| Core | github.com/mpiton/kaizen | github.com/kzn-tools/kaizen |
| Zed | (dans monorepo) | github.com/kzn-tools/zed |
| VS Code | (dans monorepo) | github.com/kzn-tools/vscode |
| Cloud | (n'existe pas) | github.com/kzn-tools/cloud |
| npm | npmjs.com/package/kzn-cli | (inchangé) |

---

## Timeline Suggérée

| Semaine | Action |
|---------|--------|
| S1 | Créer organisation + repos vides |
| S2-S4 | Développer système de licensing dans monorepo actuel |
| S5 | Transférer vers kzn-tools/kaizen |
| S6 | Extraire extensions |
| S7+ | Développer cloud backend |
