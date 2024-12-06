import os
import json

CONFIG_FILE = 'config.json'

def check_and_create_config():
    if not os.path.exists(CONFIG_FILE):
        with open(CONFIG_FILE, 'w') as file:
            json.dump({"streams": []}, file)  # Crée un fichier de configuration vide avec une clé "streams"
        print(f"Fichier de configuration créé : {CONFIG_FILE}")
    else:
        with open(CONFIG_FILE, 'r') as file:
            config = json.load(file)
            if "streams" not in config:
                config["streams"] = []
                with open(CONFIG_FILE, 'w') as file:
                    json.dump(config, file, indent=4)
        print(f"Fichier de configuration trouvé : {CONFIG_FILE}")

def load_config():
    """Charge la configuration à partir du fichier config.json."""
    try:
        with open('config.json', 'r') as f:
            data = json.load(f)
            return data.get('streams', [])
    except FileNotFoundError:
        # Si le fichier n'existe pas, retourner une liste vide
        return []
    except json.JSONDecodeError:
        # En cas d'erreur de décodage, retourner une liste vide
        return []

def save_config(config):
    """Sauvegarde la configuration dans le fichier config.json."""
    with open('config.json', 'w') as f:
        json.dump({'streams': config}, f, indent=4)
