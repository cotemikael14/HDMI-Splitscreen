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
