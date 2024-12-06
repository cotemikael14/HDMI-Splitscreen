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
