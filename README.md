# HDMI-Splitscreen (version Rust)

Split screen pour simulateur : deux cartes de capture USB affichées côte à côte, avec le moins de lag possible.

## Obtenir le .exe

### Option A : GitHub le compile pour toi (le plus simple)
1. Mets ces fichiers dans ton repo (en gardant le dossier `.github/workflows`).
2. Va dans l'onglet **Actions** du repo. La compilation part toute seule à chaque push sur `main` (ou clique sur **Run workflow**).
3. Quand c'est vert ✅, clique sur la compilation et télécharge **hdmi-splitscreen-windows** en bas de la page. Le .exe est dans le zip.

### Option B : compiler sur ton PC
1. Installe Rust : https://rustup.rs (accepte les choix par défaut, ça installe aussi les outils Visual Studio).
2. Dans le dossier du projet : `cargo build --release`
3. Le programme se trouve dans `target\release\hdmi-splitscreen.exe`

## Utilisation
- `config.json` est créé à côté du .exe. Ton ancien `config.json` Python fonctionne tel quel.
- **F11** : plein écran · **Échap** : sortir du plein écran.
- En plein écran, monte la souris en haut de l'écran pour voir le menu.
- **Fichier → Paramètres** : choisir les sources, la qualité (720p/1080p, 30/60 fps), et le V-Sync.

## Pourquoi c'est plus rapide que la version Python
| | Python | Rust |
|---|---|---|
| Thread de capture | boucle à 100 % CPU | bloque en attendant la prochaine image (0 % à vide) |
| Redimensionnement | LANCZOS sur le CPU à chaque image | fait par la carte graphique (gratuit) |
| Affichage | timer Tk toutes les 10 ms | redessine dès qu'une image arrive |
| Format | 640×480, sans choix de format | 720p, meilleur format détecté (YUYV si 60 fps possible, sinon MJPEG) |
| Décodage MJPEG 720p | OpenCV + conversions | ~1,3 ms par image |

Pour le moins de lag possible : laisse le **V-Sync désactivé** (option par défaut).
