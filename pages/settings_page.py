import tkinter as tk
from tkinter import ttk, simpledialog
import json
from PIL import Image, ImageTk
from pygrabber.dshow_graph import FilterGraph
from scrip.config import load_config, save_config

class StreamStatusIndicator(ttk.Frame):
    """Widget pour afficher un indicateur de statut pour le flux."""
    def __init__(self, master, status, **kwargs):
        super().__init__(master, **kwargs)
        self.canvas = tk.Canvas(self, width=20, height=20, bg='white', highlightthickness=0)
        self.canvas.pack()
        self.update_status(status)

    def update_status(self, status):
        """Met à jour la couleur de l'indicateur en fonction du statut."""
        color = 'green' if status else 'red'
        self.canvas.delete('all')
        self.canvas.create_oval(5, 5, 15, 15, fill=color, outline=color)

class StreamSettingsDialog(simpledialog.Dialog):
    """Boîte de dialogue pour les paramètres des flux vidéo."""
    def __init__(self, parent, stream, title=None):
        self.stream = stream
        self.graph = FilterGraph()
        self.devices = self.graph.get_input_devices()

        # Debug: afficher le contenu de self.devices
        print("Devices:", self.devices)
        
        self.device_names = self.devices
        self.device_ids = list(range(len(self.device_names)))  # Utiliser des indices comme identifiant unique
        
        super().__init__(parent, title=title)

    def body(self, master):
        """Crée les champs de saisie pour les paramètres des flux."""
        self.name_label = ttk.Label(master, text="Nom:")
        self.name_label.grid(row=0, column=0)
        self.name_entry = ttk.Entry(master)
        self.name_entry.grid(row=0, column=1)
        self.name_entry.insert(0, self.stream['name'])

        self.source_label = ttk.Label(master, text="Source:")
        self.source_label.grid(row=1, column=0)
        self.source_combobox = ttk.Combobox(master, values=[f"{name} (ID: {idx})" for idx, name in enumerate(self.device_names)])
        self.source_combobox.grid(row=1, column=1)
        self.source_combobox.set(f"{self.device_names[self.stream['source']]} (ID: {self.stream['source']})")
        self.source_combobox.bind("<<ComboboxSelected>>", self.update_port_info)

        self.port_info_label = ttk.Label(master, text="")
        self.port_info_label.grid(row=2, column=0, columnspan=2)
        self.update_port_info()

        self.display_name_label = ttk.Label(master, text="Nom à afficher:")
        self.display_name_label.grid(row=3, column=0)
        self.display_name_entry = ttk.Entry(master)
        self.display_name_entry.grid(row=3, column=1)
        self.display_name_entry.insert(0, self.stream['display_name'])

        self.enabled_var = tk.BooleanVar(value=self.stream['enabled'])
        self.enabled_check = ttk.Checkbutton(master, text="Activer", variable=self.enabled_var)
        self.enabled_check.grid(row=4, column=0, columnspan=2)

    def update_port_info(self, event=None):
        """Met à jour les informations sur le port USB du périphérique sélectionné."""
        selected = self.source_combobox.get()
        if selected:
            try:
                device_id = int(selected.split("ID: ")[-1].split(")")[0].strip())
                port_info = f"Port USB utilisé : {device_id}"
            except ValueError:
                port_info = "Port USB utilisé : N/A"
        else:
            port_info = "Port USB utilisé : N/A"
        self.port_info_label.config(text=port_info)

    def apply(self):
        """Applique les modifications aux paramètres des flux."""
        selected = self.source_combobox.get()
        if selected:
            try:
                device_id = int(selected.split("ID: ")[-1].split(")")[0].strip())
                self.stream['source'] = device_id
            except ValueError:
                self.stream['source'] = 0  # Définir une valeur par défaut si l'ID ne peut pas être extrait
        self.stream['name'] = self.name_entry.get()
        self.stream['display_name'] = self.display_name_entry.get()
        self.stream['enabled'] = self.enabled_var.get()

def create_settings_page(root, close_settings_callback):
    """Crée la page des paramètres pour la gestion des flux vidéo."""
    settings_window = tk.Toplevel(root)
    settings_window.title("Paramètres")
    settings_window.geometry("500x300")

    streams = load_config()

    def add_stream():
        """Ajoute un nouveau flux vidéo."""
        stream = {'name': 'New Stream', 'source': 0, 'display_name': 'HDMI', 'enabled': True}
        streams.append(stream)
        refresh_streams_list()

    def delete_stream(index):
        """Supprime un flux vidéo."""
        del streams[index]
        refresh_streams_list()

    def edit_stream(index):
        """Édite les paramètres d'un flux vidéo."""
        stream = streams[index]
        dialog = StreamSettingsDialog(settings_window, stream, title="Modifier le flux")
        refresh_streams_list()

    def refresh_streams_list():
        """Actualise la liste des flux vidéo affichés."""
        for widget in streams_frame.winfo_children():
            widget.destroy()

        for index, stream in enumerate(streams):
            frame = ttk.Frame(streams_frame)
            frame.pack(fill=tk.X, pady=5)

            status_indicator = StreamStatusIndicator(frame, stream['enabled'])
            status_indicator.pack(side=tk.LEFT, padx=5)

            ttk.Label(frame, text=stream['name']).pack(side=tk.LEFT, padx=5)
            ttk.Button(frame, text="Modifier", command=lambda idx=index: edit_stream(idx)).pack(side=tk.LEFT, padx=5)
            ttk.Button(frame, text="X", command=lambda idx=index: delete_stream(idx)).pack(side=tk.LEFT, padx=5)

    streams_frame = ttk.Frame(settings_window)
    streams_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)

    add_button = ttk.Button(settings_window, text="Ajouter un flux", command=add_stream)
    add_button.pack(pady=5)

    save_button = ttk.Button(settings_window, text="Enregistrer", command=lambda: [save_config(streams), close_settings_callback(), settings_window.destroy()])
    save_button.pack(pady=5)

    refresh_streams_list()
