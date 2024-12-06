# interface.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\interface.py
import tkinter as tk
from pages.home_page import create_home_page
from pages.settings_page import create_settings_page
from pages.about_page import show_about

def create_interface(root):
    fullscreen = [False]

    def toggle_fullscreen(event=None):
        fullscreen[0] = not fullscreen[0]
        root.attributes("-fullscreen", fullscreen[0])
        update_menu_text()

    def exit_fullscreen(event=None):
        """Sortir du mode plein écran."""
        if fullscreen[0]:
            toggle_fullscreen()

    def update_menu_text():
        if fullscreen[0]:
            view_menu.entryconfig(0, label="Quitter le mode plein écran")
            root.config(menu="")
        else:
            view_menu.entryconfig(0, label="Plein écran")
            root.config(menu=menu_bar)

    def show_menu_bar(event=None):
        if fullscreen[0]:
            root.config(menu=menu_bar)

    def hide_menu_bar(event=None):
        if fullscreen[0]:
            root.config(menu="")

    def show_settings(refresh_streams_callback):
        create_settings_page(root, refresh_streams_callback)

    menu_bar, view_menu = create_home_page(
        root, 
        lambda refresh_callback: show_settings(refresh_callback), 
        toggle_fullscreen, 
        show_about
    )
    root.config(menu=menu_bar)

    root.bind("<F11>", toggle_fullscreen)
    root.bind("<Escape>", exit_fullscreen)
    root.bind("<Motion>", lambda event: show_menu_bar() if event.y < 50 else hide_menu_bar())


# main.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\main.py
from tkinter import Tk
from PIL import Image, ImageTk
from interface import create_interface
from scrip.config import check_and_create_config

def main():
    check_and_create_config()  # Vérifie et crée le fichier de configuration si nécessaire
    
    root = Tk()
    root.title("Interface de gestion des flux")
    root.geometry("900x700")
    
    # Définir l'icône de l'application
    try:
        root.iconbitmap('LOGO.ico')
    except Exception as e:
        print(f"Erreur lors du chargement de l'icône : {e}")
    
    create_interface(root)
    root.mainloop()

if __name__ == "__main__":
    main()


# about_page.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\pages\about_page.py
from tkinter import Toplevel, Label, Button

def show_about():
    """Affiche une fenêtre 'À propos'."""
    about_window = Toplevel()
    about_window.grab_set()
    about_window.overrideredirect(True)
    #about_window.overrideredirect(1)  # Supprime les bordures de fenêtre
    about_window.title("À propos")
    
    Label(about_window, text="Version 0.01", font=("Arial", 14)).pack(pady=10)
    Label(about_window, text="Date de la version : 2024-07-26", font=("Arial", 12)).pack(pady=10)
    Label(about_window, text="Créateur : Mikaël Côté", font=("Arial", 12)).pack(pady=10)
    
    Button(about_window, text="Fermer", command=about_window.destroy).pack(pady=10)


# home_page.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\pages\home_page.py
import tkinter as tk
import json
import cv2
from PIL import Image, ImageTk
from scrip.video_capture import VideoCapture
from scrip.config import load_config
from scrip.background import setup_background

def create_home_page(root, show_settings, toggle_fullscreen, show_about):
    """Crée la page d'accueil avec le menu et les flux vidéo."""
    menu_bar = tk.Menu(root)
    
    def on_closing():
        """Ferme proprement l'application en arrêtant tous les flux vidéo."""
        for cap, _, _ in caps:
            cap.stop()
        root.quit()
        root.destroy()

    # Menu Fichier
    file_menu = tk.Menu(menu_bar, tearoff=0)
    file_menu.add_command(label="Paramètres", command=lambda: show_settings(refresh_streams))
    file_menu.add_command(label="Quitter", command=on_closing)
    menu_bar.add_cascade(label="Fichier", menu=file_menu)

    # Menu Affichage
    view_menu = tk.Menu(menu_bar, tearoff=0)
    view_menu.add_command(label="Plein écran", command=toggle_fullscreen)
    menu_bar.add_cascade(label="Affichage", menu=view_menu)

    # Menu Aide
    help_menu = tk.Menu(menu_bar, tearoff=0)
    help_menu.add_command(label="À propos", command=show_about)
    menu_bar.add_cascade(label="Aide", menu=help_menu)

    root.config(menu=menu_bar)
    root.grid_rowconfigure(0, weight=1)  # Ensure the first row takes all vertical space

    video_frames = []

    # Configurer l'image d'arrière-plan
    update_background, background_label = setup_background(root)

    def refresh_streams():
        """Actualise les flux vidéo en fonction de la configuration."""
        nonlocal video_frames

        for frame, _, _ in video_frames:
            frame.grid_forget()
        video_frames = []

        if background_label:
            background_label.grid_forget()

        streams = load_config()

        active_streams = [stream for stream in streams if stream['enabled']]
        print(f"Flux actifs : {active_streams}")

        num_active_streams = len(active_streams)

        if num_active_streams == 0:
             update_background()
             return

        if background_label:
            background_label.grid_forget()

        root.update_idletasks()  # Ensure the window size is updated
        column_width = root.winfo_width() // num_active_streams

        for idx, stream in enumerate(active_streams):
            frame = tk.Frame(root)
            frame.grid(row=0, column=idx, sticky="nsew")
            frame.config(bg="black")

            # Label for the stream name
            label = tk.Label(frame, text=stream['display_name'], bg="black", fg="white")
            label.pack(side=tk.TOP, fill=tk.X)

            video_label = tk.Label(frame, text="", bg="black", fg="white")
            video_label.pack(fill=tk.BOTH, expand=True)
            video_frames.append((frame, video_label, stream))

            # Set the width for each frame
            frame.config(width=column_width)
            video_label.config(width=column_width)

        # Ensure all columns are weighted equally
        for i in range(num_active_streams):
            root.columnconfigure(i, weight=1)

        start_video_capture()

    caps = []

    def start_video_capture():
        """Démarre la capture vidéo pour chaque flux actif."""
        nonlocal caps
        for cap, _, _ in caps:
            cap.stop()
        caps = []

        for frame, video_label, stream in video_frames:
            cap = VideoCapture(stream['source'])
            caps.append((cap, video_label, stream['display_name']))
            cap.start()
            print(f"Flux {stream['source']} démarré.")

    def update_video():
        """Met à jour les images vidéo capturées et les affiche."""
        all_streams_available = False

        for cap, video_label, display_name in caps:
            ret, frame = cap.read()
            if ret:
                all_streams_available = True
                frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
                img = Image.fromarray(frame)
                img = img.resize((video_label.winfo_width(), video_label.winfo_height()), Image.LANCZOS)
                imgtk = ImageTk.PhotoImage(image=img)
                video_label.imgtk = imgtk
                video_label.config(image=imgtk, text='')
            else:
                video_label.config(image='', text=f"Aucun flux vidéo disponible ({display_name})")

        if not all_streams_available:
            for frame, video_label, _ in video_frames:
                video_label.config(text="Aucun flux vidéo disponible", image='')

        root.after(10, update_video)

    root.protocol("WM_DELETE_WINDOW", on_closing)
    
    refresh_streams()
    update_video()

    return menu_bar, view_menu


# settings_page.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\pages\settings_page.py
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


# background.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\scrip\background.py
import tkinter as tk
from PIL import Image, ImageTk

def setup_background(root):
    """Configure l'image d'arrière-plan et gère les mises à jour."""
    background_image = None
    background_label = None

    def update_background():
        nonlocal background_image, background_label
        
        if background_label:
            background_label.grid_forget()

        try:
            # Calculer les dimensions avec une marge de 10%
            window_width = root.winfo_width()
            window_height = root.winfo_height()
            margin_percentage = 0.1
            new_width = int(window_width * (1 - margin_percentage))
            new_height = int(window_height * (1 - margin_percentage))

            background_image = Image.open("LOGO.png")  # Remplacez par le chemin de votre image
            background_image = background_image.resize(
                (new_width, new_height), 
                Image.Resampling.LANCZOS  # Utilisez Image.Resampling.LANCZOS au lieu de ANTIALIAS
            )
            background_photo = ImageTk.PhotoImage(background_image)
            background_label = tk.Label(root, image=background_photo)
            background_label.image = background_photo  # Keep a reference
            background_label.place(
                x=(window_width - new_width) // 2, 
                y=(window_height - new_height) // 2,
                width=new_width,
                height=new_height
            )
        except Exception as e:
            print(f"Erreur lors du chargement de l'image d'arrière-plan : {e}")

    return update_background, background_label


# config.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\scrip\config.py
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


# video_capture.py  C:\Users\cotem\Documents\PlatformIO\Projects\HDMI - Copie\scrip\video_capture.py
import cv2
from threading import Thread

class VideoCapture:
    def __init__(self, source=0):
        self.source = source
        self.cap = cv2.VideoCapture(source)
        self.ret = False
        self.frame = None
        self.stopped = False
        self.thread = None

    def start(self):
        """Start the video capture in a new thread."""
        self.stopped = False
        self.thread = Thread(target=self.update)
        self.thread.start()
        return self

    def update(self):
        """Continuously capture frames from the video source."""
        while not self.stopped:
            self.ret, self.frame = self.cap.read()

    def read(self):
        """Return the latest captured frame."""
        return self.ret, self.frame

    def stop(self):
        """Stop the video capture and release resources."""
        self.stopped = True
        if self.thread:
            self.thread.join()
        self.cap.release()


