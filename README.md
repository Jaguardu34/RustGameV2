# RustGameV2

Jeu 3D interactif développé en **Rust** avec le moteur **Bevy** et le physics engine **Avian3D**.

## Features

- **Monde 3D** — scène interactive avec camera libre (free camera)
- **Système de building** — placer et organiser des objets dans le monde
- **Contrôleur joueur** — mouvement et interactions
- **Éditeur intégré** — modification de la scène en temps réel
- **Gizmos de transformation** — déplacement, rotation, scale d'objets avec UI de configuration
- **Inspector** — inspection des propriétés des entités sélectionnées
- **Sauvegarde** — persistance de la scène via `moonshine-save`
- **Outils de dev** — overlay FPS, free camera pour le débogage

## Stack technique

- **Rust** (édition 2024)
- **Bevy 0.19** — moteur ECS 3D
- **Avian3d 0.7** — physics engine 3D
- **Moonshine-save** — système de sauvegarde sérialisée
- **Feathers** — UI légère pour Bevy

## Installation

```bash
git clone https://github.com/Jaguardu34/RustGameV2.git
cd RustGameV2
cargo run --release
```

## Structure

```
src/
├── main.rs           # Entry point du jeu
├── editor.rs         # Plugin éditeur de scène
├── editor_ui/        # Interface de l'éditeur
│   ├── gizmo_config.rs  # Config des gizmos de transformation
│   └── inspector.rs     # Inspector d'entités
├── building.rs       # Système de placement de bâtiments
├── player.rs         # Contrôleur du joueur
├── resources.rs      # Gestion des ressources globales
└── scene.rs          # Gestion de la scène
```

## License

Private — tous droits réservés.
